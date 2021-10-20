pub const NYQ: f32 = 48_000.0;
pub const BUFFER_SIZE: usize = 64;
pub type Buffer = [f32; BUFFER_SIZE];

pub const FFT_SIZE: usize = 64;
const FFT_BUFFERS: usize = FFT_SIZE / BUFFER_SIZE;
const FFT_IMSIZE: usize = FFT_SIZE * 2;
pub type FFT = [f32; FFT_SIZE];

mod client;
mod analyze;
mod ringbuf;

pub use client::Jack as Audio;
pub use analyze::prelude::*;