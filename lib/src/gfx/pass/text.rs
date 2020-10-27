use std::collections::HashMap;

use wgpu_glyph::{ab_glyph::FontArc, FontId, GlyphBrush, GlyphBrushBuilder, Section, Text};

use crate::gfx::frame::Frame;
use crate::gfx::wgpu;
use crate::math::{Vector2, Vector4};
use crate::resource;

pub struct TextBuilder<'a, 'b> {
    pass: &'a TextPass,
    text: Text<'b>,
}

impl<'a, 'b> TextBuilder<'a, 'b> {
    fn new(pass: &'a TextPass, text: &'b str) -> Self {
        Self {
            pass,
            text: Text::new(text),
        }
    }

    pub fn font(mut self, font: &str) -> Self {
        self.text = self.text.with_font_id(self.pass.alias[font]);
        self
    }

    pub fn color(mut self, color: Vector4) -> Self {
        self.text = self.text.with_color(color);
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.text = self.text.with_scale(scale);
        self
    }
}

pub struct DrawBuilder<'a, 'b> {
    pass: &'a TextPass,
    section: Section<'b>,
}

impl<'a, 'b> DrawBuilder<'a, 'b> {
    pub fn new(pass: &'a TextPass) -> Self {
        Self {
            pass,
            section: Section::default(),
        }
    }

    pub fn at(mut self, pos: Vector2) -> Self {
        self.section = self.section.with_screen_position((pos.x, pos.y));
        self
    }

    pub fn text<F: FnOnce(TextBuilder<'a, 'b>) -> TextBuilder<'a, 'b>>(
        mut self,
        text: &'b str,
        f: F,
    ) -> Self {
        self.section = self
            .section
            .add_text(f(TextBuilder::new(self.pass, text)).text);
        self
    }
}

pub struct TextPassBuilder {
    fonts: Vec<FontArc>,
    alias: HashMap<String, FontId>,
}

impl TextPassBuilder {
    pub fn new() -> Self {
        Self {
            fonts: vec![],
            alias: HashMap::new(),
        }
    }

    pub fn with(mut self, alias: &str, file: &str) -> Self {
        self.fonts.push(resource::read_font(file));
        self.alias
            .insert(alias.to_owned(), FontId(self.fonts.len()));
        self
    }

    pub fn build(self, device: &wgpu::Device) -> TextPass {
        TextPass::new(device, self)
    }
}

pub struct TextPass {
    alias: HashMap<String, FontId>,
    brush: GlyphBrush<()>,
}

impl TextPass {
    fn new(device: &wgpu::Device, builder: TextPassBuilder) -> Self {
        let brush = GlyphBrushBuilder::using_fonts(builder.fonts)
            .build(device, wgpu::defaults::texture_format());

        Self {
            alias: builder.alias,
            brush,
        }
    }

    pub fn draw<F: for<'a, 'b> FnOnce(DrawBuilder<'a, 'b>) -> DrawBuilder<'a, 'b>>(
        &mut self,
        f: F,
    ) {
        let section = f(DrawBuilder::new(self)).section;
        self.brush.queue(section);
    }

    pub fn encode(&mut self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        self.brush
            .draw_queued(
                &frame.app.device,
                &mut frame.app.staging,
                frame.encoder.as_mut().unwrap(),
                target,
                1920,
                1080,
            )
            .unwrap();
    }
}
