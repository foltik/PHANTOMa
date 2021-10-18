use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

use crate::Model;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Bubbles {
    pub color: [f32; 3],
    pub t: f32,
    pub w: f32,
    pub h: f32,
    pub dx: f32,
    pub freq: f32,
}

pub struct BubblesPass {
    synth: SynthPass,
    uniform: UniformStorage<Bubbles>,
}

impl BubblesPass {
    pub fn new(device: &wgpu::Device, bubbles: Bubbles) -> Self {
        let uniform = UniformStorage::new(device, "bubbles", bubbles);
        let synth = SynthPass::new(device, "bubbles", "bubbles.frag.spv", Some(&uniform.uniform));
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

impl Deref for BubblesPass {
    type Target = Bubbles;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl DerefMut for BubblesPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}