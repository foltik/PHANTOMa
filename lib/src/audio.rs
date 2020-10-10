use jack::{AudioIn, Port};
use ringbuf::{Consumer, Producer, RingBuffer};
use rustfft::{num_complex::Complex32, num_traits::Zero, FFTplanner};
use std::cmp::Ordering::Less;
use std::thread;

// TODO: Make these adjustable with jack::NotificationProcessor instead of hard coding.. :thinking:
pub const NYQ: f32 = 48_000.0;
const FRAME_SIZE: usize = 1024;
const FRAME_QUEUE_SIZE: usize = 64;
pub type Frame = [f32; FRAME_SIZE];
const FRAME_EMPTY: Frame = [0.0; FRAME_SIZE];

pub const FFT_SIZE: usize = 2048;
const FFT_IMSIZE: usize = FFT_SIZE * 2;
const FFT_QUEUE_SIZE: usize = 16;

pub type FFT = [f32; FFT_SIZE];
const FFT_EMPTY: FFT = [0.0; FFT_SIZE];

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

pub struct AudioClient {
    #[allow(dead_code)]
    // client: jack::AsyncClient<N, P>,
    samples: Frame,
    samples_rx: Consumer<Frame>,
    fft: FFT,
    fft_rx: Consumer<FFT>,
}

pub trait Audio {
    fn update(&mut self);

    fn fft(&self) -> &FFT;
    fn samples(&self) -> &Frame;

    fn rms(&self) -> f32 {
        rms(self.fft())
    }

    fn rms_range(&self, f0: f32, f1: f32) -> f32 {
        let (i, j) = (bin(f0), bin(f1));
        rms(&self.fft()[i..j])
    }

    fn peak(&self) -> f32 {
        peak(self.samples())
    }
}

impl Audio for AudioClient {
    fn update(&mut self) {
        if !self.samples_rx.is_empty() {
            drain(&mut self.samples_rx, &mut self.samples);
        }

        if !self.fft_rx.is_empty() {
            drain(&mut self.fft_rx, &mut self.fft);
        }
    }

    fn fft(&self) -> &FFT {
        &self.fft
    }

    fn samples(&self) -> &Frame {
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
    let jack_analyze_buffer = RingBuffer::<Frame>::new(FRAME_QUEUE_SIZE);
    let (mut jack_analyze_tx, jack_analyze_rx) = jack_analyze_buffer.split();

    // Create a ringbuffer for sending raw samples from the JACK processing thread to the main thread
    let jack_main_buffer = RingBuffer::<Frame>::new(FRAME_QUEUE_SIZE);
    let (mut jack_main_tx, jack_main_rx) = jack_main_buffer.split();

    // Create a ringbuffer for sending FFT data from the analysis thread back to the main thread
    let fft_buffer = RingBuffer::<FFT>::new(FFT_QUEUE_SIZE);
    let (fft_tx, fft_rx) = fft_buffer.split();

    thread::spawn(move || {
        // Create the JACK processing thread
        let mut process_buffer = FRAME_EMPTY;
        let process = jack::ClosureProcessHandler::new(
            move |j: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
                process(
                    j,
                    ps,
                    &in_left,
                    &in_right,
                    &mut process_buffer,
                    &mut jack_analyze_tx,
                    &mut jack_main_tx,
                )
            },
        );

        // Activate the JACK processing thread
        let _client = client.activate_async(Notifications, process).unwrap();

        loop {
            thread::park();
        }
    });

    // Create the analysis thread
    thread::spawn(move || analyze(jack_analyze_rx, fft_tx));

    AudioClient {
        // client,
        fft_rx,
        samples: FRAME_EMPTY,
        samples_rx: jack_main_rx,
        fft: FFT_EMPTY,
    }
}

pub fn transmit<T: Copy>(tx: &mut Producer<T>, t: &T) {
    if tx.is_full() {
        return;
    }

    let n = tx.push_slice(std::slice::from_ref(t));
    assert_eq!(n, 1, "transmit: failed to push slice");
}

pub fn receive<T: Copy>(rx: &mut Consumer<T>, t: &mut T) {
    while rx.is_empty() {
        thread::sleep(std::time::Duration::from_millis(1));
    }

    *t = rx.pop().unwrap();
}

pub fn drain<T: Copy>(rx: &mut Consumer<T>, t: &mut T) {
    while rx.is_empty() {
        thread::sleep(std::time::Duration::from_millis(1));
    }

    while rx.len() > 1 {
        rx.pop().unwrap();
    }

    *t = rx.pop().unwrap();
}

pub fn process(
    _j: &jack::Client,
    ps: &jack::ProcessScope,
    in_left: &Port<AudioIn>,
    in_right: &Port<AudioIn>,
    buffer: &mut Frame,
    analyze_tx: &mut Producer<Frame>,
    main_tx: &mut Producer<Frame>,
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

    transmit(analyze_tx, buffer);
    transmit(main_tx, buffer);

    jack::Control::Continue
}

pub fn analyze(mut rx: Consumer<Frame>, mut fft_tx: Producer<FFT>) {
    // Set up buffers for the input, complex FFT I/O, and result
    let mut buffer: [Frame; 4] = [FRAME_EMPTY; 4];
    let mut complex_in = vec![Complex32::zero(); FFT_IMSIZE];
    let mut complex_out = vec![Complex32::zero(); FFT_IMSIZE];
    let mut result = FFT_EMPTY;

    // Set up the FFT
    let mut planner = FFTplanner::<f32>::new(false);
    let fft = planner.plan_fft(FFT_IMSIZE);

    // Set up the window and calculate the factor we need to scale the FFT result by
    let window: Vec<_> = apodize::hanning_iter(FFT_SIZE).map(|v| v as f32).collect();
    let window_factor = window.iter().map(|x| *x as f32).sum::<f32>();

    // This *shouldn't* have any allocations
    loop {
        buffer.iter_mut().for_each(|frame| receive(&mut rx, frame));
        let flat: [f32; FRAME_SIZE * 4] = unsafe { std::mem::transmute(buffer) };

        // Copy the samples into the real parts of the complex buffer and apply the window function
        flat
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
        transmit(&mut fft_tx, &result);

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
