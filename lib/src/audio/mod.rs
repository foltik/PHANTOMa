pub const NYQ: f32 = 48_000.0;

pub const FRAME_SIZE: usize = 1024;
pub type Frame = [f32; FRAME_SIZE];

pub const FFT_SIZE: usize = 2048;
const FFT_IMSIZE: usize = FFT_SIZE * 2;
pub type FFT = [f32; FFT_SIZE];

mod client;
mod midi;
mod analyze;
mod ringbuf;

pub use client::Jack as Audio;
pub use analyze::prelude::*;
pub use midi::{MidiBank, Midi};