use super::Device;

pub mod gen;

pub mod types;
use types::*;

#[derive(Copy, Clone, Debug)]
pub struct LaunchpadX {
    mode: Mode,
}


#[derive(Copy, Clone, Debug)]
pub enum Input {
    Press(Index, f32),
    Release(Index),

    MonoPressure(f32),
    PolyPressure(Index, f32),

    Up(bool),
    Down(bool),
    Left(bool),
    Right(bool),
    Session(bool),
    Note(bool),
    Custom(bool),
    Capture(bool),

    Volume(bool),
    Pan(bool),
    A(bool),
    B(bool),
    Stop(bool),
    Mute(bool),
    Solo(bool),
    Record(bool),

    Unknown,
}

#[derive(Copy, Clone, Debug)]
pub enum Output {
    Light(Pos, Color),
    Flash(Pos, Color),
    Pulse(Pos, Color),
    Off(Pos),
    Rgb(Pos, (u8, u8, u8)),
    Clear,
    ClearAll,

    Mode(Mode),
    Brightness(f32),
    Velocity(Velocity),
    Pressure(Pressure, PressureCurve),

    Clock,
}

fn float(v: u8) -> f32 {
    (v as f32) / 127.0
}

fn byte(f: f32) -> u8 {
    (f.clamp(0.0, 1.0) * 127.0) as u8
}

impl Device for LaunchpadX {
    type Input = Input;
    type Output = Output;

    fn new() -> Self {
        Self {
            mode: Mode::Live,
        }
    }

    fn process_input(&mut self, raw: &[u8]) -> Option<Input> {
        Some(match raw[0] {
            0x90 => {
                match self.mode {
                    Mode::Live => Input::Unknown,
                    Mode::Programmer => {
                        let i = Index::from_byte(raw[1]);
                        match raw[2] {
                            0 => Input::Release(i),
                            v => Input::Press(i, float(v))
                        }
                    }
                }
            },
            0x80 => {
                match raw[2] {
                    0x40 => match self.mode {
                        Mode::Live => Input::Unknown,
                        Mode::Programmer => {
                            let i = Index::from_byte(raw[1]);
                            Input::Release(i)
                        }
                    },
                    _ => Input::Unknown,
                }
            }
            0xB0 => {
                let b = raw[2] == 0x7F;
                match raw[1] {
                    0x5B => Input::Up(b),
                    0x5C => Input::Down(b),
                    0x5D => Input::Left(b),
                    0x5E => Input::Right(b),
                    0x5F => Input::Session(b),
                    0x60 => Input::Note(b),
                    0x61 => Input::Custom(b),
                    0x62 => Input::Capture(b),
                    0x59 => Input::Volume(b),
                    0x4F => Input::Pan(b),
                    0x45 => Input::A(b),
                    0x3B => Input::B(b),
                    0x31 => Input::Stop(b),
                    0x27 => Input::Mute(b),
                    0x1D => Input::Solo(b),
                    0x13 => Input::Record(b),
                    _ => unreachable!(),
                }
            }
            0xD0 => Input::MonoPressure(float(raw[1])),
            0xA0 => Input::PolyPressure(Index::from_byte(raw[1]), float(raw[2])),
            _ => return None
        })
    }

    fn process_output(&mut self, output: Output) -> Vec<u8> {
        match output {
            Output::Light(p, col) => vec![0x90, p.byte(), col.byte()],
            Output::Flash(p, col) => vec![0x91, p.byte(), col.byte()],
            Output::Pulse(p, col) => vec![0x92, p.byte(), col.byte()],
            Output::Off(p) => vec![0x80, p.byte(), 0x0],
            Output::Rgb(p, col) => vec![0xF0, 0x0, 0x20, 0x29, 0x2, 0xC, 0x3, 0x3, p.byte(), col.0, col.1, col.2, 0xF7],
            Output::Clear => {
                let mut data = Vec::with_capacity(8 + (64 * 3));

                data.extend_from_slice(&[0xF0, 0x0, 0x20, 0x29, 0x2, 0xC, 0x3]);
                for i in 0..64 {
                    data.extend_from_slice(&[0x0, Index(i).byte(), 0x0]);
                }
                data.push(0xF7);

                data
            }
            Output::ClearAll => {
                let mut data = Vec::with_capacity(8 + (81 * 3));

                data.extend_from_slice(&[0xF0, 0x0, 0x20, 0x29, 0x2, 0xC, 0x3]);
                for i in 0..81 {
                    data.extend_from_slice(&[0x0, Index9(i).byte(), 0x0]);
                }
                data.push(0xF7);

                data
            }
            Output::Mode(m) => {
                self.mode = m;
                let mode = match m {
                    Mode::Live => 0,
                    Mode::Programmer => 1,
                };
                vec![0xF0, 0x00, 0x20, 0x29, 0x2, 0x0C, 0x0E, mode, 0xF7]
            },
            Output::Brightness(f) => vec![0xF0, 0x00, 0x20, 0x29, 0x2, 0xC, 0x8, byte(f), 0xF7],
            Output::Velocity(v) => {
                let curve = match v {
                    Velocity::Low => 0,
                    Velocity::Medium => 1,
                    Velocity::High => 2,
                    Velocity::Fixed(_) => 3,
                };

                let fixed = match v {
                    Velocity::Fixed(v) => v,
                    _ => 0x00
                };

                vec![0xF0, 0x0, 0x20, 0x29, 0x2, 0xC, 0x04, curve, fixed, 0xF7]
            },
            Output::Pressure(a, t) => {
                let ty = match a {
                    Pressure::Polyphonic => 0,
                    Pressure::Channel => 1,
                    Pressure::Off => 2,
                };

                let thres = match t {
                    PressureCurve::Low => 0,
                    PressureCurve::Medium => 1,
                    PressureCurve::High => 2,
                };

                vec![0xF0, 0x0, 0x20, 0x29, 0x2, 0xC, 0xB, ty, thres, 0xF7]
            },
            Output::Clock => vec![0xF8],
        }
    }
}