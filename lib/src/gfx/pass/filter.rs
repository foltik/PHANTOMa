use crate::gfx::{uniform::Uniform, wgpu};

const BILLBOARD_SHADER: &str = "../resources/shaders/billboard.vert.spv";
const COMPOSITE_SHADER: &str = "../resources/shaders/composite.frag.spv";

pub struct FilterPass {
    views: Vec<wgpu::RawTextureView>,
    image_group: wgpu::BindGroup,
    uniform_group: Option<wgpu::BindGroup>,
    pipeline: wgpu::RenderPipeline,
}

impl FilterPass {
    pub fn new<T: Copy>(
        device: &wgpu::Device,
        label: &str,
        fragment: &str,
        uniform: Option<&Uniform<T>>,
    ) -> Self {
        Self::new_composite(device, label, 1, Some(fragment), uniform)
    }

    pub fn new_composite<T: Copy>(
        device: &wgpu::Device,
        label: &str,
        n: u32,
        fragment: Option<&str>,
        uniform: Option<&Uniform<T>>,
    ) -> Self {
        let vs = crate::resource::read_shader(device, BILLBOARD_SHADER);
        let fs = crate::resource::read_shader(device, fragment.unwrap_or(COMPOSITE_SHADER));

        let (image_group, image_layout, views) = Self::image_group(device, label, n);

        let uniform_groups =
            uniform.map(|u| Self::uniform_group(device, label, u));

        let pipeline = Self::pipeline(
            device,
            label,
            &vs,
            &fs,
            &image_layout,
            uniform_groups.as_ref().map(|g| &g.1),
        );

        Self {
            views,
            image_group,
            uniform_group: uniform_groups.map(|g| g.0),
            pipeline,
        }
    }

    fn pipeline(
        device: &wgpu::Device,
        label: &str,
        vs: &wgpu::ShaderModule,
        fs: &wgpu::ShaderModule,
        image_layout: &wgpu::BindGroupLayout,
        uniform_layout: Option<&wgpu::BindGroupLayout>,
    ) -> wgpu::RenderPipeline {
        if let Some(uniform_layout) = uniform_layout {
            wgpu::util::PipelineBuilder::new(label)
                .with_layout(&image_layout)
                .with_layout(&uniform_layout)
                .render(&vs)
                .fragment_shader(&fs)
                .build(device)
        } else {
            wgpu::util::PipelineBuilder::new(label)
                .with_layout(&image_layout)
                .render(&vs)
                .fragment_shader(&fs)
                .build(device)
        }
    }

    fn uniform_group<T: Copy>(
        device: &wgpu::Device,
        label: &str,
        uniform: &Uniform<T>,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        let uniform_layout =
            wgpu::util::BindGroupLayoutBuilder::new(&format!("{}_uniform", label))
                .uniform(wgpu::ShaderStage::FRAGMENT, uniform)
                .build(device);

        let uniform_group = wgpu::util::BindGroupBuilder::new(&format!("{}_uniform", label))
            .uniform(uniform)
            .build(device, &uniform_layout);

        (uniform_group, uniform_layout)
    }

    fn image_group(
        device: &wgpu::Device,
        label: &str,
        n: u32,
    ) -> (
        wgpu::BindGroup,
        wgpu::BindGroupLayout,
        Vec<wgpu::RawTextureView>,
    ) {
        assert_ne!(n, 0, "Tried to create a filter pass with 0 images!");

        let views = (0..n)
            .map(|i| {
                wgpu::util::TextureBuilder::new(&format!("{}_input_{}", label, i))
                    .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
                    .build(device)
                    .view()
                    .build()
            })
            .collect::<Vec<_>>();

        let sampler = wgpu::util::SamplerBuilder::new(&format!("{}_sampler", label)).build(device);

        if n > 1 {
            let count = Uniform::new(device, &format!("{}_tex_count", label), Some(&n));

            let image_layout =
                wgpu::util::BindGroupLayoutBuilder::new(&format!("{}_image", label))
                    .textures(wgpu::ShaderStage::FRAGMENT, &views)
                    .sampler(wgpu::ShaderStage::FRAGMENT)
                    .uniform(wgpu::ShaderStage::FRAGMENT, &count)
                    .build(device);

            let views = views.into_iter().map(|v| v.view).collect::<Vec<_>>();

            let image_group = wgpu::util::BindGroupBuilder::new(&format!("{}_image", label))
                .textures(&views)
                .sampler(&sampler)
                .uniform(&count)
                .build(device, &image_layout);

            (image_group, image_layout, views)
        } else {
            let image_layout =
                wgpu::util::BindGroupLayoutBuilder::new(&format!("{}_image", label))
                    .texture(wgpu::ShaderStage::FRAGMENT, &views[0])
                    .sampler(wgpu::ShaderStage::FRAGMENT)
                    .build(device);

            let image_group = wgpu::util::BindGroupBuilder::new(&format!("{}_image", label))
                .texture(&views[0])
                .sampler(&sampler)
                .build(device, &image_layout);

            let views = views.into_iter().map(|v| v.view).collect::<Vec<_>>();

            (image_group, image_layout, views)
        }
    }

    pub fn view(&self, n: usize) -> &wgpu::RawTextureView {
        &self.views[n]
    }

    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::RawTextureView) {
        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(target, |b| b)
            .begin(encoder);

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.image_group, &[]);
        if let Some(uniform_group) = self.uniform_group.as_ref() {
            pass.set_bind_group(1, uniform_group, &[]);
        }

        pass.draw(0..3, 0..1);
    }
}
