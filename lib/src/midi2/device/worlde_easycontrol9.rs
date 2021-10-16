use super::Device;

#[derive(Copy, Clone, Debug)]
pub struct WorldeEasyControl9 {
    bank: u8,
    encoder: u8,
}

#[derive(Copy, Clone, Debug)]
pub enum Input {
    TopButton(u8, bool),
    MainButton(u8, bool),
    CtrlButton(u8, bool),
    BankButton(u8, bool),
    Slider(u8, f32),
    Knob(u8, f32),
    Fader(f32),
    Encoder(bool),
    Bank(u8),
}

impl Device for WorldeEasyControl9 {
    type Input = Input;

    fn new() -> Self {
        Self {
            bank: 0,
            encoder: 0,
        }
    }

    fn process_input(&mut self, raw: &[u8]) -> Option<Input> {
        Some(match raw[0] {
            176..=179 => {
                let cc = raw[1];
                let state = raw[2];
                let on = state == 127;
                let fl = state as f32 / 126.0;

                match cc {
                    1..=2 => Input::BankButton(cc - 1, on),
                    9 => Input::Fader(
                        (std::cmp::max(state, 1) - 1) as f32 / 126.0,
                    ),
                    14..=22 => Input::Knob(cc - 14, fl),
                    23..=31 => Input::MainButton(cc - 23, on),
                    32..=40 => Input::Slider(cc - 32, fl),
                    44..=49 => Input::CtrlButton(cc - 44, on),
                    67 => Input::TopButton(0, on),
                    64 => Input::TopButton(1, on),
                    _ => unreachable!(),
                }
            }
            192 => {
                let v = raw[1];
                use std::cmp::Ordering;
                match v.cmp(&self.encoder) {
                    Ordering::Greater => {
                        self.encoder = v;
                        Input::Encoder(true)
                    }
                    Ordering::Less => {
                        self.encoder = v;
                        Input::Encoder(false)
                    }
                    Ordering::Equal => match self.encoder {
                        0 => Input::Encoder(false),
                        127 => Input::Encoder(true),
                        _ => unreachable!(),
                    },
                }
            }
            240 => {
                let b = raw[9];
                self.bank = b;
                Input::Bank(b)
            }
            _ => unreachable!()
        })
    }

    fn process_output(&mut self, output: Vec<u8>) -> Vec<u8> {
        output
    }
}