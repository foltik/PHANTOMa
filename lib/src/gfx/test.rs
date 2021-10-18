use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

use crate::Model;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Test {
    pub color: [f32; 3],
    pub w: f32,
    pub h: f32,
    pub t: f32,
    pub fov: f32,
    pub r: f32,
    pub glow: f32,
    pub thickness: f32,
}

pub struct TestPass {
    synth: SynthPass,
    uniform: UniformStorage<Test>,
}

impl TestPass {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = UniformStorage::new(device, "test", Test {
            color: [1.0, 1.0, 1.0],
            w: 1920.0,
            h: 1080.0,
            t: 0.0,
            fov: 0.5,
            r: 1.0,
            glow: 1.0, 
            thickness: 0.4,
        });
        let synth = SynthPass::new(device, "test", "torus.frag.spv", Some(&uniform.uniform));
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

impl Deref for TestPass {
    type Target = Test;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl DerefMut for TestPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}