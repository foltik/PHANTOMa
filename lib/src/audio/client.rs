use std::sync::Arc;
use std::thread;

use crossbeam_queue::ArrayQueue;

use super::analyze;
use super::ringbuf::{self, Consumer, Producer, RingBuffer};
use super::{Frame, FFT, FFT_SIZE, FRAME_SIZE};

use super::midi::{Midi, MidiState, MidiBank, MidiRaw};

const FRAME_QUEUE_SIZE: usize = 64;
const FFT_QUEUE_SIZE: usize = 16;

pub struct Jack {
    pub samples: Frame,
    pub fft: FFT,
    midi: MidiState,

    midi_rx: Arc<ArrayQueue<MidiRaw>>,
    samples_rx: Consumer<Frame>,
    fft_rx: Consumer<FFT>,
}

impl Jack {
    pub fn update(&mut self) {
        if !self.samples_rx.is_empty() {
            ringbuf::drain(&mut self.samples_rx, &mut self.samples);
        }

        if !self.fft_rx.is_empty() {
            ringbuf::drain(&mut self.fft_rx, &mut self.fft);
        }
    }

    pub fn midi(&mut self) -> Vec<(MidiBank, Midi)> {
        let mut messages = Vec::new();

        while let Some(raw) = self.midi_rx.pop() {
            messages.push(self.midi.process(raw));
        }

        messages
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

impl Default for Jack {
    fn default() -> Self {
        // Create JACK client
        let (client, _status) = jack::Client::new("PHANTOMa", jack::ClientOptions::NO_START_SERVER)
            .expect("Failed to connect to JACK audio server!");

        // Register audio ports
        let in_left = client
            .register_port("in_left", jack::AudioIn::default())
            .unwrap();
        let in_right = client
            .register_port("in_right", jack::AudioIn::default())
            .unwrap();

        let in_midi = client
            .register_port("midi", jack::MidiIn::default())
            .unwrap();

        // // Create a queue for sending MIDI messages
        let midi_rx = Arc::new(ArrayQueue::<MidiRaw>::new(128));
        let midi_tx = Arc::clone(&midi_rx);

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
            let mut process_buffer = [0.0; FRAME_SIZE];
            let process = jack::ClosureProcessHandler::new(
                move |j: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
                    process(
                        j,
                        ps,
                        &in_left,
                        &in_right,
                        &in_midi,
                        &midi_tx,
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
        thread::spawn(move || analyze::analyze(jack_analyze_rx, fft_tx));

        Self {
            midi: MidiState::default(),
            midi_rx,
            fft_rx,
            samples: [0.0; FRAME_SIZE],
            samples_rx: jack_main_rx,
            fft: [0.0; FFT_SIZE],
        }
    }
}

pub fn process(
    _j: &jack::Client,
    ps: &jack::ProcessScope,
    in_left: &jack::Port<jack::AudioIn>,
    in_right: &jack::Port<jack::AudioIn>,
    in_midi: &jack::Port<jack::MidiIn>,
    midi_tx: &Arc<ArrayQueue<MidiRaw>>,
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

    in_midi.iter(ps).for_each(|m| {
        if !midi_tx.is_full() {
            let mut buf = [0; 16];
            for (i, b) in m.bytes.iter().take(16).enumerate() {
                buf[i] = *b;
            }
            midi_tx.push(buf).unwrap();
        }
    });

    ringbuf::transmit(analyze_tx, buffer);
    ringbuf::transmit(main_tx, buffer);

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

    fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
        log::trace!("JACK: buffer size changed to {}", sz);
        jack::Control::Continue
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        log::trace!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        log::trace!("JACK: xrun occurred");
        jack::Control::Continue
    }
}
