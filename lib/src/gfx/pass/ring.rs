use crate::gfx::frame::Frame;
use crate::gfx::wgpu;

pub struct RingPass {
    pipeline: wgpu::RenderPipeline,
    views: Vec<wgpu::TextureView>,
    groups: Vec<wgpu::BindGroup>,
    i_latest: usize,
    i_target: usize,
    n: usize,
}

impl RingPass {
    pub fn new(device: &wgpu::Device, n: usize) -> Self {
        let vs_mod = crate::resource::read_shader(device, "billboard.vert.spv");
        let fs_mod = crate::resource::read_shader(device, "passthrough.frag.spv");

        let sampler = wgpu::util::SamplerBuilder::new("ring").build(device);

        let views = (0..n)
            .map(|_| {
                wgpu::util::TextureBuilder::new_color("ring")
                    .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
                    .build(device)
                    .view()
                    .build()
            })
            .collect::<Vec<_>>();

        let layout = wgpu::util::BindGroupLayoutBuilder::new("ring")
            .texture(wgpu::ShaderStage::FRAGMENT, &views[0])
            .sampler(wgpu::ShaderStage::FRAGMENT)
            .build(device);

        let groups = views
            .iter()
            .map(|v| {
                wgpu::util::BindGroupBuilder::new("ring")
                    .texture(v)
                    .sampler(&sampler)
                    .build(device, &layout)
            })
            .collect();

        let pipeline = wgpu::util::PipelineBuilder::new("ring")
            .with_layout(&layout)
            .render(&vs_mod)
            .fragment_shader(&fs_mod)
            .build(device);

        Self {
            pipeline,
            views,
            groups,
            i_latest: 1,
            i_target: 0,
            n,
        }
    }

    pub fn get(&self, i: usize) -> &wgpu::TextureView {
        &self.views[(self.i_latest + i).rem_euclid(self.n) as usize]
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.views[self.i_target as usize]
    }

    pub fn update(&mut self) {
        self.i_latest = (self.i_latest as i32 - 1).rem_euclid(self.n as i32) as usize;
        self.i_target = (self.i_target as i32 - 1).rem_euclid(self.n as i32) as usize;
    }

    pub fn encode(&self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(target, |b| b)
            .begin(frame);

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.groups[self.i_target], &[]);
        pass.draw(0..3, 0..1);
    }
}
