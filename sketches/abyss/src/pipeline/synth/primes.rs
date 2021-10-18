
use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Primes {
    pub color: [f32; 3],
    pub t: f32,
    pub nx: f32,
    pub ny: f32,
    pub dx: f32,
    pub dy: f32,
    pub twin: f32,
    pub op: u32,
}

pub struct PrimesPass {
    synth: SynthPass,
    uniform: UniformStorage<Primes>,
}

impl PrimesPass {
    pub fn new(device: &wgpu::Device, primes: Primes) -> Self {
        let uniform = UniformStorage::new(device, "primes", primes);
        let synth = SynthPass::new(device, "primes", "primes.frag.spv", Some(&uniform.uniform));
        Self {
            synth,
            uniform,
        }
    }

    pub fn update(&mut self, t: f32) {
        self.uniform.t = t;
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.uniform.upload(frame);
        self.synth.encode(frame, view);
    }
}

impl Deref for PrimesPass {
    type Target = Primes;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl DerefMut for PrimesPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}