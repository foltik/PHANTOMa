use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct IsoTri {
    pub color: [f32; 3],
    pub aspect: f32,
    pub t: f32,
    pub r: f32,
    pub weight: f32,
    pub thickness: f32,
}

pub struct IsoTriPass {
    synth: SynthPass,
    uniform: UniformStorage<IsoTri>,
}

impl IsoTriPass {
    pub fn new(device: &wgpu::Device, isotri: IsoTri) -> Self {
        let uniform = UniformStorage::new(device, "isotri", isotri);
        let synth = SynthPass::new(device, "isotri", "isotri.frag.spv", Some(&uniform.uniform));
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

impl Deref for IsoTriPass {
    type Target = IsoTri;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl DerefMut for IsoTriPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}