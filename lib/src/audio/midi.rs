
#[derive(Debug, Clone, Copy)]
pub enum Midi {
    TopButton(u8, bool),
    MainButton(u8, bool),
    CtrlButton(u8, bool),
    BankButton(u8, bool),
    Slider(u8, f32),
    Knob(u8, f32),
    Fader(f32),
    Encoder(i8),
    Bank(u8),
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum MidiBank {
    B0,
    B1,
    B2,
    B3,
}

pub(crate) type MidiRaw = [u8; 16];

pub(crate) struct MidiState {
    encoder: u8,
    bank: MidiBank,
}

impl Default for MidiState {
    fn default() -> Self {
        MidiState {
            encoder: 0,
            bank: MidiBank::B0,
        }
    }
}

impl MidiState {
    pub fn process(&mut self, raw: MidiRaw) -> (MidiBank, Midi) {
        let message = match raw[0] {
            176..=179 => {
                let cc = raw[1];
                let state = raw[2];
                let on = state == 127;
                let fl = state as f32 / 126.0;

                match cc {
                    1..=2 => Midi::BankButton(cc - 1, on),
                    9 => Midi::Fader((std::cmp::max(state, 1) - 1) as f32 / 126.0),
                    14..=22 => Midi::Knob(cc - 14, fl),
                    23..=31 => Midi::MainButton(cc - 23, on),
                    32..=40 => Midi::Slider(cc - 32, fl),
                    44..=49 => Midi::CtrlButton(cc - 44, on),
                    67 => Midi::TopButton(0, on),
                    64 => Midi::TopButton(1, on),
                    _ => Midi::Unknown,
                    // _ => Midi::Unknown(Vec::from(raw)),
                }
            }
            192 => {
                let v = raw[1];
                use std::cmp::Ordering;
                match v.cmp(&self.encoder) {
                    Ordering::Greater => {
                        self.encoder = v;
                        Midi::Encoder(1)
                    }
                    Ordering::Less => {
                        self.encoder = v;
                        Midi::Encoder(-1)
                    }
                    Ordering::Equal => match self.encoder {
                        0 => Midi::Encoder(-1),
                        127 => Midi::Encoder(1),
                        _ => panic!("Invalid MIDI encoder state"),
                    },
                }
            }
            240 => {
                let b = raw[9];
                self.bank = match b {
                    0 => MidiBank::B0,
                    1 => MidiBank::B1,
                    2 => MidiBank::B2,
                    3 => MidiBank::B3,
                    _ => panic!("Unknown MIDI bank"),
                };
                Midi::Bank(b)
            }
            _ => Midi::Unknown,
            // _ => Midi::Unknown(Vec::from(raw)),
        };

        (self.bank, message)
    }
}
