use std::ops::{Deref, DerefMut};

use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Test {
    t: f32,
}

pub struct ToggleSynthPass<T: Copy = ()> {
    active: bool,
    synth: SynthPass,
    uniform: Option<UniformStorage<T>>,
}

impl <T: Copy> ToggleSynthPass<T> {
    pub fn new(device: &wgpu::Device, uniform: Option<T>, active: bool) -> Self {
        let uniform = uniform.map(|u| UniformStorage::new(device, "toggle", u));
        let synth = SynthPass::new(device, "fill", "loading.frag.spv", match uniform {
            Some(ref u) => Some(&u.uniform),
            _ => None
        });
        Self {
            active,
            synth,
            uniform,
        }
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        if self.active {
            if let Some(uniform) = self.uniform.as_ref() {
                uniform.upload(frame);
            }
            self.synth.encode(frame, view);
        }
    }

    pub fn active(&mut self, active: bool) {
        self.active = active;
    }
}

impl <T: Copy> Deref for ToggleSynthPass<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.uniform.as_ref().unwrap()
    }
}

impl <T: Copy> DerefMut for ToggleSynthPass<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.uniform.as_mut().unwrap()
    }
}