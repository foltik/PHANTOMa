use crate::gfx::{uniform::Uniform, wgpu};

const BILLBOARD_SHADER: &str = "../resources/shaders/billboard.vert.spv";

pub struct CompositePass {
    views: Vec<wgpu::wgpu::TextureView>,
    uniform: Uniform<u32>,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl CompositePass {
    pub fn new(device: &wgpu::Device, label: &'static str, n: u32, fragment: Option<&str>) -> Self {
        let vs_mod = crate::resource::read_shader(device, BILLBOARD_SHADER);
        let fs_mod = crate::resource::read_shader(device, fragment.unwrap_or("composite.frag.spv"));

        let uniform = Uniform::new(device, &format!("{}_tex_count", label), Some(&n));

        let views = (0..n).map(|i|
            wgpu::util::TextureBuilder::new(&format!("{}_input_{}", label, i))
                .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
                .build(device)
                .view()
                .build()
        ).collect::<Vec<_>>();

        let sampler = wgpu::util::SamplerBuilder::new(&format!("{}_sampler", label)).build(device);

        let bind_group_layout = wgpu::util::BindGroupLayoutBuilder::new(label)
            .textures(wgpu::ShaderStage::FRAGMENT, &views)
            .sampler(wgpu::ShaderStage::FRAGMENT)
            .uniform(wgpu::ShaderStage::FRAGMENT, &uniform)
            .build(device);

        let views = views.into_iter().map(|v| v.view).collect::<Vec<_>>();

        let bind_group = wgpu::util::BindGroupBuilder::new(label)
            .textures(&views)
            .sampler(&sampler)
            .uniform(&uniform)
            .build(device, &bind_group_layout);

        let pipeline = wgpu::util::PipelineBuilder::new(label)
            .with_layout(&bind_group_layout)
            .render(&vs_mod)
            .fragment_shader(&fs_mod)
            .build(device);

        Self {
            views,
            uniform,
            bind_group,
            pipeline,
        }
    }

    pub fn view(&self, n: usize) -> &wgpu::wgpu::TextureView {
        &self.views[n]
    }

    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(target, |b| b)
            .begin(encoder);

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);

        pass.draw(0..3, 0..1);
    }
}
