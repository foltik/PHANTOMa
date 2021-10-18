use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

use crate::Model;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Torus {
    pub color: [f32; 3],
    pub w: f32,
    pub h: f32,
    pub t: f32,
    pub fov: f32,
    pub r: f32,
    pub glow: f32,
    pub thickness: f32,
}

pub struct TorusPass {
    synth: SynthPass,
    uniform: UniformStorage<Torus>,
}

impl TorusPass {
    pub fn new(device: &wgpu::Device, torus: Torus) -> Self {
        let uniform = UniformStorage::new(device, "torus", torus);
        let synth = SynthPass::new(device, "torus", "torus.frag.spv", Some(&uniform.uniform));
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

impl Deref for TorusPass {
    type Target = Torus;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl DerefMut for TorusPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}