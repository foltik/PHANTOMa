use std::thread;
use anyhow::Result;

use super::analyze;
use super::ringbuf::{self, Consumer, Producer, RingBuffer};
use super::{Buffer, FFT, FFT_SIZE, BUFFER_SIZE};

const FRAME_QUEUE_SIZE: usize = 64;
const FFT_QUEUE_SIZE: usize = 16;

pub struct Jack {
    pub samples: Buffer,
    pub fft: FFT,

    samples_rx: Consumer<Buffer>,
    fft_rx: Consumer<FFT>,
}

impl Jack {
    pub fn open() -> Result<Self> {
        // Create JACK client
        let (client, _status) = jack::Client::new("PHANTOMa", jack::ClientOptions::NO_START_SERVER)?;

        // Register audio ports
        let in_left = client
            .register_port("in_left", jack::AudioIn::default())?;
        let in_right = client
            .register_port("in_right", jack::AudioIn::default())?;

        // Create a ringbuffer for sending raw samples from the JACK processing thread to the analysis thread
        let jack_analyze_buffer = RingBuffer::<Buffer>::new(FRAME_QUEUE_SIZE);
        let (mut jack_analyze_tx, jack_analyze_rx) = jack_analyze_buffer.split();

        // Create a ringbuffer for sending raw samples from the JACK processing thread to the main thread
        let jack_main_buffer = RingBuffer::<Buffer>::new(FRAME_QUEUE_SIZE);
        let (mut jack_main_tx, jack_main_rx) = jack_main_buffer.split();

        // Create a ringbuffer for sending FFT data from the analysis thread back to the main thread
        let fft_buffer = RingBuffer::<FFT>::new(FFT_QUEUE_SIZE);
        let (fft_tx, fft_rx) = fft_buffer.split();

        thread::spawn(move || {
            // Create the JACK processing thread
            let mut process_buffer = [0.0; BUFFER_SIZE];
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
            let _client = client.activate_async(Notifications, process)
                .expect("Failed to active JACK processing thread");

            loop {
                thread::park();
            }
        });

        // Create the analysis thread
        thread::spawn(move || analyze::analyze(jack_analyze_rx, fft_tx));

        Ok(Self {
            fft_rx,
            samples: [0.0; BUFFER_SIZE],
            samples_rx: jack_main_rx,
            fft: [0.0; FFT_SIZE],
        })
    }

    pub fn maybe_open() -> Option<Self> {
        match Self::open() {
            Ok(a) => Some(a),
            Err(e) => {
                log::warn!("Failed to open JACK: {:?}", e);
                None
            }
        }
    }

    pub fn update(&mut self) {
        if !self.samples_rx.is_empty() {
            ringbuf::drain(&mut self.samples_rx, &mut self.samples);
        }

        if !self.fft_rx.is_empty() {
            ringbuf::drain(&mut self.fft_rx, &mut self.fft);
        }
    }

    pub fn rms(&self) -> f32 {
        analyze::rms(&self.fft)
    }

    pub fn rms_range(&self, f0: f32, f1: f32) -> f32 {
        let (i, j) = (analyze::bin(f0), analyze::bin(f1));
        analyze::rms(&self.fft[i..j])
    }

    pub fn peak(&self) -> f32 {
        analyze::peak(&self.samples)
    }
}

pub fn process(
    _j: &jack::Client,
    ps: &jack::ProcessScope,
    in_left: &jack::Port<jack::AudioIn>,
    in_right: &jack::Port<jack::AudioIn>,
    buffer: &mut Buffer,
    analyze_tx: &mut Producer<Buffer>,
    main_tx: &mut Producer<Buffer>,
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
    
    if !analyze_tx.is_full() {
        ringbuf::transmit(analyze_tx, buffer);
    }

    if !main_tx.is_full() {
        ringbuf::transmit(main_tx, buffer);
    }

    jack::Control::Continue
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        log::trace!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status,
            reason
        );
    }

    // fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
    //     log::trace!("JACK: buffer size changed to {}", sz);
    //     jack::Control::Continue
    // }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        log::trace!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        log::trace!("JACK: xrun occurred");
        jack::Control::Continue
    }
}
