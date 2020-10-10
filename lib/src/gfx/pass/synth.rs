use crate::gfx::{wgpu, uniform::Uniform};

const BILLBOARD_SHADER: &str = "../resources/shaders/billboard.vert.spv";

pub struct SynthPass {
    uniform_group: Option<wgpu::BindGroup>,
    pipeline: wgpu::RenderPipeline,
}

impl SynthPass {
    pub fn new<T: Copy + Clone>(device: &wgpu::Device, label: &'static str, fragment: &str, uniform: Option<&Uniform<T>>) -> Self {
        let shader = |p| crate::resource::read_shader(device, p);
        let vs_mod = shader(BILLBOARD_SHADER);
        let fs_mod = shader(fragment);

        if let Some(uniform) = uniform {
            let uniform_layout = wgpu::util::BindGroupLayoutBuilder::new(&format!("{}_uniforms", label))
                .uniform(wgpu::ShaderStage::FRAGMENT, uniform)
                .build(device);

            let uniform_group = wgpu::util::BindGroupBuilder::new(&format!("{}_uniforms", label))
                .uniform(uniform)
                .build(device, &uniform_layout);
            
            let pipeline = wgpu::util::PipelineBuilder::new(label)
                .with_layout(&uniform_layout)
                .render(&vs_mod)
                .fragment_shader(&fs_mod)
                .build(device);

            Self {
                uniform_group: Some(uniform_group),
                pipeline,
            }
        } else {
            let pipeline = wgpu::util::PipelineBuilder::new(label)
                .render(&vs_mod)
                .fragment_shader(&fs_mod)
                .build(device);

            Self {
                uniform_group: None,
                pipeline,
            }
        }
    }

    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::wgpu::TextureView) {
        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(target, |b| b)
            .begin(encoder);

        pass.set_pipeline(&self.pipeline);

        if let Some(uniform_group) = self.uniform_group.as_ref() {
            pass.set_bind_group(0, uniform_group, &[]);
        }

        pass.draw(0..3, 0..1);
    }
}