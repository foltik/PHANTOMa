use crate::component::ComponentState;
use rustfft::{num_complex::Complex32, num_traits::Zero, FFTplanner};
use spsc_bip_buffer::{bip_buffer_with_len, BipBufferReader, BipBufferWriter};
use std::sync::{Arc, Mutex};
use std::thread;

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
        println!("JACK: buffer size changed to {}", sz);
        jack::Control::Continue
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
        println!(
            "JACK: {} latency has changed",
            match mode {
                jack::LatencyType::Capture => "capture",
                jack::LatencyType::Playback => "playback",
            }
        );
    }
}

pub fn fft(samples: &[f32]) -> Vec<f32> {
    let len = samples.len();
    let mut complex: Vec<Complex32> = samples.iter().map(|s| Complex32::new(*s, 0.0)).collect();

    let mut res: Vec<Complex32> = vec![Complex32::zero(); len];

    let mut plan = FFTplanner::new(false);
    let fft = plan.plan_fft(len);
    fft.process(&mut complex, &mut res);

    res.iter()
        .take(len / 2)
        .map(|&c| (c.norm_sqr().sqrt() * 2.0) / (len as f32 * 2.0))
        .collect()
}

pub fn rms(samples: &[f32]) -> f32 {
    let sum: f32 = samples.iter().map(|s| s.abs().powi(2)).sum();
    (sum / samples.len() as f32).sqrt()
}

pub fn init(
    state: Arc<Mutex<ComponentState>>,
) -> jack::AsyncClient<impl jack::NotificationHandler, impl jack::ProcessHandler> {
    let (client, _status) =
        jack::Client::new("PHANTOMa", jack::ClientOptions::NO_START_SERVER).unwrap();

    let in_left = client
        .register_port("in_left", jack::AudioIn::default())
        .unwrap();
    let in_right = client
        .register_port("in_right", jack::AudioIn::default())
        .unwrap();

    let (mut tx, mut rx) = bip_buffer_with_len(65536);

    let process = jack::ClosureProcessHandler::new(
        move |j: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            process(j, ps, Arc::clone(&state), tx)
        },
    );

    let analyze = thread::spawn(move || {
        analyze(Arc::clone(&state), rx);
    });

    let active_client = client.activate_async(Notifications, process).unwrap();

    active_client
}

pub fn process(
    j: &jack::Client,
    ps: &jack::ProcessScope,
    state: Arc<Mutex<ComponentState>>,
    mut tx: BipBufferWriter,
) -> jack::Control {
    let raw_left = in_left.as_slice(ps);
    let raw_right = in_right.as_slice(ps);

    let mono: Vec<f32> = raw_left
        .iter()
        .zip(raw_right.iter())
        .map(|(&x, &y)| (x + y) / 2.0)
        .collect();

    let raw: &[u8] = unsafe { std::slice::from_raw_parts(mono.as_ptr() as *const u8, mono.len()) };

    let mut res = tx.spin_reserve(mono.len());
    res.copy_from_slice(raw);
    res.send();

    //let rate = j.sample_rate();
    //let size = j.buffer_size();
    //let t = size as f32 / rate as f32;

    let bins = fft(&mono);
    let amp = rms(&mono);

    {
        let mut state = state.lock().unwrap();

        state.amp = amp;

        if state.fft.len() != bins.len() {
            state.fft.resize(bins.len(), 0.0);
        }
        state.fft.copy_from_slice(&bins);
    }

    jack::Control::Continue
}

pub fn analyze(state: Arc<Mutex<ComponentState>>, mut rx: BipBufferReader) {
    loop {
        while rx.valid().len() < 512 {}
    }
}
