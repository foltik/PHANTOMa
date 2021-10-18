use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Starfield {
    pub color: [f32; 3],
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub t: f32,
    pub speed: f32,
    pub warp: f32,
}

pub struct StarfieldPass {
    synth: SynthPass,
    uniform: UniformStorage<Starfield>,
}

impl StarfieldPass {
    pub fn new(device: &wgpu::Device, starfield: Starfield) -> Self {
        let uniform = UniformStorage::new(device, "starfield", starfield);
        let synth = SynthPass::new(device, "starfield", "starfield.frag.spv", Some(&uniform.uniform));
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

impl Deref for StarfieldPass {
    type Target = Starfield;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl DerefMut for StarfieldPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}