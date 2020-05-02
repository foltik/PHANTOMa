use jack::{AudioIn, Port};
use ringbuf::{Consumer, Producer, RingBuffer};
use rustfft::{num_complex::Complex32, num_traits::Zero, FFTplanner};
use std::cmp::Ordering::Less;
use std::fmt::Debug;
use std::thread;

// TODO: Make these adjustable with jack::NotificationProcessor instead of hard coding.. :thinking:
pub const NYQ: f32 = 48_000.0;
const BUFFER_SIZE: usize = 1024;
const BUFFER_BYTES: usize = BUFFER_SIZE * std::mem::size_of::<f32>();
const BUFFER_RETENTION: usize = 64;

pub const FFT_SIZE: usize = 2048;
const FFT_IMSIZE: usize = FFT_SIZE * 2;
pub const FFT_FSIZE: f32 = FFT_SIZE as f32;
const FFT_BYTES: usize = FFT_SIZE * std::mem::size_of::<f32>();
const FFT_RETENTION: usize = 64;

pub fn freq(bin: usize) -> f32 {
    bin as f32 * (NYQ / FFT_SIZE as f32 / 2.0)
}

pub fn bin(freq: f32) -> usize {
    (freq / (NYQ / FFT_SIZE as f32 / 2.0)).floor() as usize
}

pub fn rms(bins: &[f32]) -> f32 {
    let sum: f32 = bins.iter().map(|s| s.abs().powi(2)).sum();
    (sum / bins.len() as f32).sqrt()
}

pub fn peak(samples: &[f32]) -> f32 {
    samples
        .iter()
        .map(|s| s.abs())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Less))
        .unwrap()
}

pub fn dbfs(v: f32) -> f32 {
    20.0 * (v + 0.0001).log10()
}

pub struct AudioClient<N, P> {
    #[allow(dead_code)]
    client: jack::AsyncClient<N, P>,
    samples: Vec<f32>,
    samples_rx: Consumer<u8>,
    fft: Vec<f32>,
    fft_rx: Consumer<u8>,
}

pub trait Audio {
    fn update(&mut self);

    fn fft(&self) -> &Vec<f32>;
    fn samples(&self) -> &Vec<f32>;

    fn rms(&self) -> f32 {
        rms(&self.fft())
    }

    fn rms_range(&self, f0: f32, f1: f32) -> f32 {
        let (i, j) = (bin(f0), bin(f1));
        rms(&self.fft()[i..j])
    }

    fn peak(&self) -> f32 {
        peak(&self.samples())
    }
}

impl<N, P> Audio for AudioClient<N, P> {
    fn update(&mut self) {
        receive_buffer(&mut self.samples_rx, &mut self.samples);
        receive_buffer(&mut self.fft_rx, &mut self.fft);
    }

    fn fft(&self) -> &Vec<f32> {
        &self.fft
    }

    fn samples(&self) -> &Vec<f32> {
        &self.samples
    }
}

// TODO: replace AsyncClient with wrapper so Audio doesn't have to be boxed
pub fn init() -> impl Audio {
    // Create JACK client
    let (client, _status) =
        jack::Client::new("PHANTOMa", jack::ClientOptions::NO_START_SERVER).unwrap();

    // Register audio ports
    let in_left = client
        .register_port("in_left", jack::AudioIn::default())
        .unwrap();
    let in_right = client
        .register_port("in_right", jack::AudioIn::default())
        .unwrap();

    // Create a ringbuffer for sending raw samples from the JACK processing thread to the analysis thread
    let jack_analyze_buffer = RingBuffer::<u8>::new(BUFFER_BYTES * BUFFER_RETENTION);
    let (mut jack_analyze_tx, jack_analyze_rx) = jack_analyze_buffer.split();

    // Create a ringbuffer for sending raw samples from the JACK processing thread to the main thread
    let jack_main_buffer = RingBuffer::<u8>::new(BUFFER_BYTES * BUFFER_RETENTION);
    let (mut jack_main_tx, jack_main_rx) = jack_main_buffer.split();

    // Create a ringbuffer for sending FFT data from the analysis thread back to the main thread
    let fft_buffer = RingBuffer::<u8>::new(FFT_BYTES * FFT_RETENTION);
    let (fft_tx, fft_rx) = fft_buffer.split();

    // Create the JACK processing thread
    let mut process_buffer = vec![0.0; BUFFER_SIZE];
    let process = jack::ClosureProcessHandler::new(
        move |j: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            process(
                j,
                ps,
                &in_left,
                &in_right,
                &mut process_buffer,
                &mut jack_main_tx,
                &mut jack_analyze_tx,
            )
        },
    );

    // Create the analysis thread
    thread::spawn(move || analyze(jack_analyze_rx, fft_tx));

    // Activate the JACK processing thread
    let client = client.activate_async(Notifications, process).unwrap();

    AudioClient {
        client,
        fft_rx,
        samples: vec![0.0; FFT_SIZE],
        samples_rx: jack_main_rx,
        fft: vec![0.0; FFT_SIZE],
    }
}

pub fn bytes<T>(vec: &Vec<T>) -> &[u8] {
    let size = vec.capacity() * std::mem::size_of::<T>();
    unsafe { std::slice::from_raw_parts(vec.as_ptr() as *const u8, size) }
}

pub fn bytes_mut<T>(vec: &mut Vec<T>) -> &mut [u8] {
    let size = vec.capacity() * std::mem::size_of::<T>();
    unsafe { std::slice::from_raw_parts_mut(vec.as_mut_ptr() as *mut u8, size) }
}

pub fn transmit_buffer<T: Debug>(tx: &mut Producer<u8>, buffer: &Vec<T>) {
    let raw = bytes(buffer);
    let len = raw.len();
    let mut reader = std::io::Cursor::new(raw);

    let mut count = 0;
    while count != len {
        if tx.is_full() {
            thread::sleep(std::time::Duration::from_millis(1));
        } else {
            let n = tx.read_from(&mut reader, Some(len - count)).unwrap();
            count += n;
        }
    }
}

pub fn transmit<T: Copy + Debug>(tx: &mut Producer<T>, t: T) {
    loop {
        if tx.is_full() {
            thread::sleep(std::time::Duration::from_millis(1));
        } else {
            tx.push(t).unwrap();
            break;
        }
    }
}

pub fn receive_buffer<T: Debug + Default>(rx: &mut Consumer<u8>, buffer: &mut Vec<T>) {
    let raw = bytes_mut(buffer);
    let len = raw.len();
    let mut writer = std::io::Cursor::new(raw);

    let mut count = 0;
    while count != len {
        if rx.is_empty() {
            thread::sleep(std::time::Duration::from_millis(1));
        } else {
            let n = rx.write_into(&mut writer, Some(len - count)).unwrap();
            count += n;
        }
    }
}

pub fn receive<T: Copy + Debug>(rx: &mut Consumer<T>, t: &mut T) {
    loop {
        if rx.is_empty() {
            thread::sleep(std::time::Duration::from_millis(1));
        } else {
            *t = rx.pop().unwrap();
            break;
        }
    }
}

pub fn process(
    _j: &jack::Client,
    ps: &jack::ProcessScope,
    in_left: &Port<AudioIn>,
    in_right: &Port<AudioIn>,
    buffer: &mut Vec<f32>,
    analyze_tx: &mut Producer<u8>,
    main_tx: &mut Producer<u8>,
) -> jack::Control {
    let raw_left = in_left.as_slice(ps);
    let raw_right = in_right.as_slice(ps);

    raw_left
        .iter()
        .zip(raw_right.iter())
        .map(|(&x, &y)| (x + y) / 2.0)
        .zip(buffer.iter_mut())
        .for_each(|(sample, v)| {
            *v = sample;
        });

    transmit_buffer(analyze_tx, buffer);
    transmit_buffer(main_tx, buffer);

    jack::Control::Continue
}

pub fn analyze(mut rx: Consumer<u8>, mut fft_tx: Producer<u8>) {
    // Set up buffers for the input, complex FFT I/O, and result
    let mut buffer = vec![0.0; FFT_SIZE];
    let mut complex_in = vec![Complex32::zero(); FFT_IMSIZE];
    let mut complex_out = vec![Complex32::zero(); FFT_IMSIZE];
    let mut result = vec![0.0; FFT_SIZE];

    // Set up the FFT
    let mut planner = FFTplanner::<f32>::new(false);
    let fft = planner.plan_fft(FFT_IMSIZE);

    // Set up the window and calculate the factor we need to scale the FFT result by
    let window: Vec<_> = apodize::hanning_iter(FFT_SIZE).map(|v| v as f32).collect();
    let window_factor = window.iter().map(|x| *x as f32).sum::<f32>();

    // This *shouldn't* have any allocations
    loop {
        receive_buffer(&mut rx, &mut buffer);

        // Copy the samples into the real parts of the complex buffer and apply the window function
        buffer
            .iter()
            .zip(complex_in.iter_mut())
            .zip(window.iter())
            .for_each(|((sample, c), w)| c.re = *sample * *w);

        fft.process(&mut complex_in, &mut complex_out);

        // Copy the abs of each complex result scaled by the window factor into the result buffer
        complex_out
            .iter()
            .take(FFT_SIZE)
            .zip(result.iter_mut())
            .for_each(|(c, v)| {
                *v = c.norm_sqr().sqrt() / window_factor;
            });

        // Send off the FFT data
        transmit_buffer(&mut fft_tx, &result);

        /*
        let energy_time = samples.iter().map(|y| y.powi(2)).sum::<f32>() * (1.0 / NYQ);
        let energy_freq = bins.iter().map(|y| (y / NYQ).abs().powi(2)).sum::<f32>() * (NYQ / FFT_FSIZE);

        let rms_time = energy_time.sqrt();
        let rms_freq = energy_freq.sqrt();

        let dbfs = 20.0 * (rms_freq * 2.0f32.sqrt()).log10();
        */
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
