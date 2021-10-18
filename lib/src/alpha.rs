use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LineWave {
    pub color: [f32; 3],
    pub t: f32,
    pub w: f32,
    pub h: f32,
    pub n1: f32,
    pub n2: f32,
    pub dz: f32,
    pub thickness: f32,
    pub falloff: f32,
    pub n: u32
}

pub struct LineWavePass {
    synth: SynthPass,
    uniform: UniformStorage<LineWave>,
}

impl LineWavePass {
    pub fn new(device: &wgpu::Device, linewave: LineWave) -> Self {
        let uniform = UniformStorage::new(device, "linewave", linewave);
        let synth = SynthPass::new(device, "linewave", "linewave.frag.spv", Some(&uniform.uniform));
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

impl Deref for LineWavePass {
    type Target = LineWave;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl DerefMut for LineWavePass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}