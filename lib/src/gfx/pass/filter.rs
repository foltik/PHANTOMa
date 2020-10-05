use crate::gfx::{wgpu, uniform::Uniform};

const BILLBOARD_SHADER: &'static str = "../resources/shaders/billboard.vert.spv";

pub struct FilterPass {
    pub view: wgpu::TextureView,
    sampler_group: wgpu::BindGroup,
    uniform_group: Option<wgpu::BindGroup>,
    pipeline: wgpu::RenderPipeline,
}

impl FilterPass {
    pub fn new<T: Copy + Clone>(device: &wgpu::Device, label: &'static str, fragment: &str, uniform: Option<&Uniform<T>>) -> Self {
        let shader = |p| crate::resource::read_shader(device, p);
        let vs_mod = shader(BILLBOARD_SHADER);
        let fs_mod = shader(fragment);

        let texture = wgpu::util::TextureBuilder::new_color(&format!("{}_input_color", label))
            .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
            .build(device);

        let view = texture.view().build();

        let sampler = wgpu::util::SamplerBuilder::new("input").build(&device);

        let sampler_layout = wgpu::util::BindGroupLayoutBuilder::new(&format!("{}_sampler_layout", label))
            .sampled_texture(wgpu::ShaderStage::FRAGMENT, &texture)
            .sampler(wgpu::ShaderStage::FRAGMENT)
            .build(device);

        let sampler_group = wgpu::util::BindGroupBuilder::new(&format!("{}_sampler_group", label))
            .texture_view(&view)
            .sampler(&sampler)
            .build(device, &sampler_layout);

        if let Some(uniform) = uniform {
            let uniform_layout = wgpu::util::BindGroupLayoutBuilder::new(&format!("{}_uniform_layout", label))
                .uniform(wgpu::ShaderStage::FRAGMENT, uniform)
                .build(device);

            let uniform_group = wgpu::util::BindGroupBuilder::new(&format!("{}_uniform_group", label))
                .uniform(uniform)
                .build(device, &uniform_layout);

            let pipeline = wgpu::util::PipelineBuilder::new(label)
                .with_layout(&sampler_layout)
                .with_layout(&uniform_layout)
                .render(&fs_mod)
                .build(device);

            Self {
                view,
                sampler_group,
                uniform_group: Some(uniform_group),
                pipeline,
            }
        } else {
            let pipeline = wgpu::util::PipelineBuilder::new(label)
                .with_layout(&sampler_layout)
                .render(&vs_mod)
                .fragment_shader(&fs_mod)
                .build(device);

            Self {
                view,
                sampler_group,
                uniform_group: None,
                pipeline,
            }
        }
    }

    pub fn encode<V: wgpu::View>(&self, encoder: &mut wgpu::CommandEncoder, target: &V) {
        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(target.view(), |b| b)
            .begin(encoder);

        pass.set_pipeline(&self.pipeline);

        pass.set_bind_group(0, &self.sampler_group, &[]);
        if let Some(uniform_group) = self.uniform_group.as_ref() {
            pass.set_bind_group(0, uniform_group, &[]);
        }

        pass.draw(0..3, 0..1);
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}