use std::collections::HashMap;

use crate::math::Vector2;

use crate::app::App;
use crate::gfx::frame::Frame;
use crate::gfx::{wgpu, uniform::UniformStorage};

#[derive(Clone, Copy)]
#[repr(C)]
struct ImageUniform {
    pos: Vector2,
    scale: Vector2,
}

struct Image {
    uniform: UniformStorage<ImageUniform>,
    group: wgpu::BindGroup,
}

impl std::ops::Deref for Image {
    type Target = UniformStorage<ImageUniform>;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

pub struct ImagePass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,

    images: HashMap<String, Image>,
}

impl ImagePass {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = |p| crate::resource::read_shader(device, p);
        let vs_mod = shader("billboard.vert.spv");
        let fs_mod = shader("image.frag.spv");

        let sampler = wgpu::util::SamplerBuilder::new("images")
            .mag_filter(wgpu::FilterMode::Nearest)
            .build(device);

        let layout = wgpu::util::BindGroupLayoutBuilder::new("image_pass_uniforms")
            .tex(wgpu::ShaderStage::FRAGMENT)
            .sampler(wgpu::ShaderStage::FRAGMENT)
            .uniform(wgpu::ShaderStage::FRAGMENT)
            .build(device);

        let pipeline = wgpu::util::PipelineBuilder::new("image_pass")
            .with_layout(&layout)
            .render(&vs_mod)
            .fragment_shader(&fs_mod)
            .build(device);

        Self {
            pipeline,
            layout,
            sampler,
            images: HashMap::new(),
        }
    }

    pub fn with(mut self, app: &App, image: &str, alias: &str, pos: Vector2, rescale: Vector2) -> Self {
        let (tex, scale) = wgpu::util::image::load(app, &crate::resource::read_image(image));
        let view = tex.view().build();

        let uniform = UniformStorage::new(&app.device, alias, ImageUniform {
            pos,
            scale,
        });

        let group = wgpu::util::BindGroupBuilder::new(&format!("image_{}", alias))
            .texture(&view)
            .sampler(&self.sampler)
            .uniform(uniform.as_ref())
            .build(&app.device, &self.layout);

        let image = Image {
            uniform,
            group,
        };

        self.images.insert(alias.to_owned(), image);

        self
    }

    pub fn encode(&self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(target, |b| b)
            .begin(frame);

        pass.set_pipeline(&self.pipeline);

        for image in self.images.values() {
            pass.set_bind_group(0, &image.group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}