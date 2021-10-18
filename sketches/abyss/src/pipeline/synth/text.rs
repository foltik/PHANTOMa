
use std::ops::{Deref, DerefMut};

use lib::gfx::pass::{TextPass, TextPassBuilder};
use lib::gfx::frame::Frame;
use lib::gfx::wgpu;

use lib::math::v4;

use crate::{ScrollPass, ScrollDir};

pub struct CodeScrollPass {
    pub str: String,
    pub str_glitch: String,
    pub font_sz: f32,

    pub text: TextPass,
    pub scroll: ScrollPass
}

impl CodeScrollPass {
    pub fn new(device: &wgpu::Device, size: (usize, usize), y_scale: f32, str: String, font_sz: f32, dir: ScrollDir, pd: f32) -> Self {
        Self {
            str: str.clone(), 
            str_glitch: str,
            font_sz,

            text: TextPassBuilder::default()
                .size(size)
                .with("default", "go.ttf")
                .build(device),

            scroll: ScrollPass::new(device, size, dir, y_scale, pd),
        }
    }

    pub fn update(&mut self, t: f32) {
        self.scroll.update(t);

        let str = &self.str_glitch;
        let scale = self.font_sz;

        self.text.draw(|d| {
            d.text(str, |t| 
                t.scale(scale)
                 .color(v4(1.0, 1.0, 1.0, 1.0)))
        });
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.text.encode(frame, self.scroll.view());
        self.scroll.encode(frame, view);
    }
}

impl Deref for CodeScrollPass {
    type Target = ScrollPass;

    fn deref(&self) -> &Self::Target {
        &self.scroll
    }
}

impl DerefMut for CodeScrollPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scroll
    }
}