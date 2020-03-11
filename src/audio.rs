use crate::component::ComponentState;
use jack::{AudioIn, Port};
use ringbuf::{Consumer, Producer, RingBuffer};
use rustfft::{num_complex::Complex32, num_traits::Zero, FFTplanner, FFT};
use std::sync::{Arc, Mutex};
use std::thread;

pub const FFT_SIZE: usize = 256;
const BUFFER_FACTOR: usize = 2;
const BUFFER_SIZE: usize = FFT_SIZE * BUFFER_FACTOR;

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

    let buf = RingBuffer::<u8>::new(BUFFER_SIZE * 64);
    let (mut tx, rx) = buf.split();

    let process = jack::ClosureProcessHandler::new(
        move |j: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            process(j, ps, &in_left, &in_right, &mut tx)
        },
    );

    // TODO: Return a struct that has a deactivate method for this thread!
    /*let analyze = */thread::spawn(move || {
        analyze(Arc::clone(&state), rx);
    });

    let active_client = client.activate_async(Notifications, process).unwrap();

    active_client
}

pub fn process(
    _j: &jack::Client,
    ps: &jack::ProcessScope,
    in_left: &Port<AudioIn>,
    in_right: &Port<AudioIn>,
    tx: &mut Producer<u8>,
) -> jack::Control {
    let raw_left = in_left.as_slice(ps);
    let raw_right = in_right.as_slice(ps);

    let mono: Vec<f32> = raw_left
        .iter()
        .zip(raw_right.iter())
        .map(|(&x, &y)| (x + y) / 2.0)
        .collect();

    let sz = mono.len();

    let mut raw: &[u8] = unsafe { std::slice::from_raw_parts(mono.as_ptr() as *const u8, sz) };

    let mut count = 0;

    loop {
        if tx.is_full() {
            thread::sleep(std::time::Duration::from_millis(1));
        } else {
            let n = tx.read_from(&mut raw, None).unwrap();
            count += n;
            if n == 0 {
                break;
            }
        }
    }

    assert_eq!(count, mono.len());

    // TODO: Sleep?
    /*
    while tx.is_full() || tx.remaining() <= sz {
        //println!("Waiting: {} with remaining {}", if tx.is_full() { "FULL" } else { "OPEN" }, tx.remaining());
    }
    // TODO: Handle error?
    //println!("{} with remaining {}", if tx.is_full() { "FULL" } else { "OPEN" }, tx.remaining());
    let n = tx.read_from(&mut raw, Some(sz)).unwrap();
    */
    //assert_eq!(n, sz);

    jack::Control::Continue
}

pub fn fft(fft: &dyn FFT<f32>, samples: &[f32]) -> Vec<f32> {
    let len = samples.len();
    let mut complex: Vec<Complex32> = samples.iter().map(|s| Complex32::new(*s, 0.0)).collect();

    let mut res: Vec<Complex32> = vec![Complex32::zero(); len];

    //let fft = planner.plan_fft(BUFFER_SIZE);
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

pub fn analyze(state: Arc<Mutex<ComponentState>>, mut rx: Consumer<u8>) {
    let mut planner = FFTplanner::new(false);
    let fft_fn = planner.plan_fft(BUFFER_SIZE);

    loop {
        let mut bytes = Vec::with_capacity(BUFFER_SIZE);

        while bytes.len() < BUFFER_SIZE {
            if rx.is_empty() {
                thread::sleep(std::time::Duration::from_millis(1));
            } else {
                rx.write_into(&mut bytes, Some(1)).unwrap();
            }
        }

        assert_eq!(bytes.len(), BUFFER_SIZE);

        let samples =
            unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const f32, bytes.len()) };

        let bins = fft(fft_fn.as_ref(), samples);
        let amp = rms(samples);

        {
            let mut state = state.lock().unwrap();

            state.amp = amp;

            if state.fft.len() != bins.len() {
                state.fft.resize(bins.len(), 0.0);
            }
            state.fft.copy_from_slice(&bins);
        }
    }
}
