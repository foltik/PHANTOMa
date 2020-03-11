use crate::component::ComponentState;
use apodize::{hanning_iter, CosineWindowIter};
use jack::{AudioIn, Port};
use ringbuf::{Consumer, Producer, RingBuffer};
use rustfft::{num_complex::Complex32, num_traits::Zero, FFTplanner, FFT};
use std::sync::{Arc, Mutex};
use std::thread;

pub const FFT_SIZE: usize = 2048;
pub const FFT_BYTES: usize = FFT_SIZE * std::mem::size_of::<f32>();
const BUFFER_SIZE: usize = 512;
const BUFFER_BYTES: usize = BUFFER_SIZE * std::mem::size_of::<f32>();

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
    /*let analyze = */
    thread::spawn(move || {
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

    let sz = mono.len() * std::mem::size_of::<f32>();

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

    assert_eq!(count, sz);

    jack::Control::Continue
}

pub fn fft(fft: &dyn FFT<f32>, samples: &[f32]) -> Vec<f32> {
    let len = samples.len();

    let window_fn = hanning_iter;

    let window: CosineWindowIter = window_fn(len);
    let window_factor = window_fn(len).map(|x| x as f32).sum::<f32>();

    let mut complex: Vec<Complex32> = samples
        .iter()
        .zip(window)
        .map(|(x, c)| Complex32::new(*x * c as f32, 0.0))
        .collect();

    let mut res: Vec<Complex32> = vec![Complex32::zero(); len];

    fft.process(&mut complex, &mut res);

    res.iter()
        .take(len / 2)
        .map(|&c| (c.norm_sqr().sqrt() / window_factor.sqrt()))
        .collect()
}

pub fn rms(samples: &[f32]) -> f32 {
    let sum: f32 = samples.iter().map(|s| s.abs().powi(2)).sum();
    (sum / samples.len() as f32).sqrt()
}

pub fn analyze(state: Arc<Mutex<ComponentState>>, mut rx: Consumer<u8>) {
    let mut planner = FFTplanner::new(false);
    let fft_fn = planner.plan_fft(FFT_SIZE * 2);

    loop {
        let mut bytes = Vec::with_capacity(BUFFER_BYTES);

        while bytes.len() < BUFFER_BYTES {
            if rx.is_empty() {
                thread::sleep(std::time::Duration::from_millis(1));
            } else {
                rx.write_into(&mut bytes, Some(1)).unwrap();
            }
        }

        assert_eq!(bytes.len(), BUFFER_BYTES);

        bytes.extend(
            std::iter::repeat(0).take((FFT_BYTES * 2) - BUFFER_BYTES),
        );

        let samples =
            unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const f32, FFT_SIZE * 2) };

        let bins = fft(fft_fn.as_ref(), samples);
        let amp = rms(samples);

        {
            let mut state = state.lock().unwrap();

            state.amp = amp;
            state.fft.copy_from_slice(&bins);
        }
    }
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
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

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }
}

