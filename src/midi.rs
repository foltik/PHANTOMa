use crossbeam_queue::SegQueue;
use midir::{Ignore, MidiInput};
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
pub enum MidiMessage {
    TopButton(u8, bool),
    MainButton(u8, bool),
    CtrlButton(u8, bool),
    BankButton(u8, bool),
    Slider(u8, f32),
    Knob(u8, f32),
    Fader(f32),
    Encoder(i8),
    Bank(u8),
    Unknown(Vec<u8>),
}

#[derive(Copy, Clone, Debug)]
pub enum MidiBank {
    B0,
    B1,
    B2,
    B3,
}

type MidiQueue = Arc<SegQueue<(MidiBank, MidiMessage)>>;

pub struct Midi {
    queue: MidiQueue,
}

impl Midi {
    pub fn poll(&self) -> Vec<(MidiBank, MidiMessage)> {
        let mut messages = Vec::with_capacity(self.queue.len());

        while !self.queue.is_empty() {
            messages.push(self.queue.pop().unwrap());
        }

        messages
    }
}

impl Default for Midi {
    fn default() -> Self {
        let queue = Arc::new(SegQueue::new());
        let sender = Arc::clone(&queue);

        thread::spawn(move || {
            let mut midi = MidiInput::new("PHANTOMa_MIDI").unwrap();
            midi.ignore(Ignore::None);

            if midi.ports().is_empty() {
                log::trace!("No MIDI devices!");
            }

            for (i, p) in midi.ports().iter().enumerate() {
                log::trace!("Midi device {}: '{}'", i, midi.port_name(p).unwrap());
            }

            struct State {
                queue: MidiQueue,
                encoder: u8,
                bank: MidiBank,
            }
            let mut state = State {
                queue: sender,
                encoder: 0,
                bank: MidiBank::B0,
            };

            let p = &midi.ports()[1];
            log::debug!("Using device '{}'", midi.port_name(p).unwrap());
            let _conn = midi
                .connect(
                    p,
                    "midi_in",
                    move |_stamp, raw, _| {
                        log::debug!("{:?}", raw);
                        let message = match raw[0] {
                            176..=179 => {
                                let cc = raw[1];
                                let state = raw[2];
                                let on = state == 127;
                                let fl = state as f32 / 126.0;

                                match cc {
                                    1..=2 => MidiMessage::BankButton(cc - 1, on),
                                    9 => MidiMessage::Fader(
                                        (std::cmp::max(state, 1) - 1) as f32 / 126.0,
                                    ),
                                    14..=22 => MidiMessage::Knob(cc - 14, fl),
                                    23..=31 => MidiMessage::MainButton(cc - 23, on),
                                    32..=40 => MidiMessage::Slider(cc - 32, fl),
                                    44..=49 => MidiMessage::CtrlButton(cc - 44, on),
                                    67 => MidiMessage::TopButton(0, on),
                                    64 => MidiMessage::TopButton(1, on),
                                    _ => MidiMessage::Unknown(Vec::from(raw)),
                                }
                            }
                            192 => {
                                let v = raw[1];
                                use std::cmp::Ordering;
                                match v.cmp(&state.encoder) {
                                    Ordering::Greater => {
                                        state.encoder = v;
                                        MidiMessage::Encoder(1)
                                    }
                                    Ordering::Less => {
                                        state.encoder = v;
                                        MidiMessage::Encoder(-1)
                                    }
                                    Ordering::Equal => match state.encoder {
                                        0 => MidiMessage::Encoder(-1),
                                        127 => MidiMessage::Encoder(1),
                                        _ => panic!("Invalid MIDI encoder state"),
                                    },
                                }
                            }
                            240 => {
                                let b = raw[9];
                                state.bank = match b {
                                    0 => MidiBank::B0,
                                    1 => MidiBank::B1,
                                    2 => MidiBank::B2,
                                    3 => MidiBank::B3,
                                    _ => panic!("Unknown MIDI bank"),
                                };
                                MidiMessage::Bank(b)
                            }
                            _ => MidiMessage::Unknown(Vec::from(raw)),
                        };

                        state.queue.push((state.bank, message));
                    },
                    (),
                )
                .unwrap();

            loop {
                thread::park();
            }
        });

        Self { queue }
    }
}
