use crate::gfx::frame::Frame;
use crate::gfx::{uniform::Uniform, wgpu};

const BILLBOARD_SHADER: &str = "billboard.vert.spv";
const COMPOSITE_SHADER: &str = "composite.frag.spv";
const PASSTHROUGH_SHADER: &str = "passthrough.frag.spv";

// FIXME: Need to find a better way to abstract out size. Right now it's a mess.
// Make it a builder?

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
        Self::new_sized(device, label, fragment, uniform, (1920, 1080))
    }

    pub fn new_sized<T: Copy>(
        device: &wgpu::Device,
        label: &str,
        fragment: &str,
        uniform: Option<&Uniform<T>>,
        size: (usize, usize),
    ) -> Self {
        Self::new_composite_sized(device, label, 1, Some(fragment), uniform, size)
    }

    pub fn new_composite<T: Copy>(
        device: &wgpu::Device,
        label: &str,
        n: u32,
        fragment: Option<&str>,
        uniform: Option<&Uniform<T>>,
    ) -> Self {
        Self::new_composite_sized(device, label, n, fragment, uniform, (1920, 1080))
    }

    pub fn new_composite_sized<T: Copy>(
        device: &wgpu::Device,
        label: &str,
        n: u32,
        fragment: Option<&str>,
        uniform: Option<&Uniform<T>>,
        size: (usize, usize),
    ) -> Self {
        let vs = crate::resource::read_shader(device, BILLBOARD_SHADER);
        let fs = crate::resource::read_shader(device, fragment.unwrap_or(COMPOSITE_SHADER));

        let (image_group, image_layout, views) = Self::image_group(device, label, n, size);

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

    pub fn new_add<T: Copy>(
        device: &wgpu::Device,
        label: &str,
        n: u32,
        fragment: Option<&str>,
        uniform: Option<&Uniform<T>>,
    ) -> Self {
        let vs = crate::resource::read_shader(device, BILLBOARD_SHADER);
        let fs = crate::resource::read_shader(device, fragment.unwrap_or("add.frag.spv"));

        let (image_group, image_layout, views) = Self::image_group(device, label, n, (1920, 1080));

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

    pub fn new_passthrough(device: &wgpu::Device) -> Self {
        Self::new::<()>(device, "passthrough", PASSTHROUGH_SHADER, None)
    }

    pub fn new_passthrough_sized(device: &wgpu::Device, size: (usize, usize)) -> Self {
        Self::new_sized::<()>(device, "passthrough", PASSTHROUGH_SHADER, None, size)
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
                .with_layout(image_layout)
                .with_layout(uniform_layout)
                .render(vs)
                .fragment(fs)
                .build(device)
        } else {
            wgpu::util::PipelineBuilder::new(label)
                .with_layout(image_layout)
                .render(vs)
                .fragment(fs)
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
                .uniform(wgpu::ShaderStages::FRAGMENT)
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
        size: (usize, usize),
    ) -> (
        wgpu::BindGroup,
        wgpu::BindGroupLayout,
        Vec<wgpu::RawTextureView>,
    ) {
        assert_ne!(n, 0, "Tried to create a filter pass with 0 images!");

        let views = (0..n)
            .map(|i| {
                wgpu::util::TextureBuilder::new(&format!("{}_input_{}", label, i))
                    .usage(wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
                    .size([size.0 as u32, size.1 as u32, 1])
                    .build(device)
                    .view()
                    .build()
            })
            .collect::<Vec<_>>();

        let sampler = wgpu::util::SamplerBuilder::new(&format!("{}_sampler", label))
            .address_mode(wgpu::AddressMode::Repeat)
            .build(device);

        if n > 1 {
            let count = Uniform::new(device, &format!("{}_tex_count", label), Some(&n));

            let image_layout =
                wgpu::util::BindGroupLayoutBuilder::new(&format!("{}_image", label))
                    .textures(wgpu::ShaderStages::FRAGMENT, n as usize)
                    .sampler(wgpu::ShaderStages::FRAGMENT)
                    .uniform(wgpu::ShaderStages::FRAGMENT)
                    .build(device);

            let views = views.into_iter().map(|v| v.view).collect::<Vec<_>>();

            let view_refs = views.iter().collect::<Vec<_>>();
            let image_group = wgpu::util::BindGroupBuilder::new(&format!("{}_image", label))
                .textures(&view_refs)
                .sampler(&sampler)
                .uniform(&count)
                .build(device, &image_layout);

            (image_group, image_layout, views)
        } else {
            let image_layout =
                wgpu::util::BindGroupLayoutBuilder::new(&format!("{}_image", label))
                    .texture(wgpu::ShaderStages::FRAGMENT, &views[0])
                    .sampler(wgpu::ShaderStages::FRAGMENT)
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

    pub fn encode(&self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(target, |b| b)
            .begin(frame);

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.image_group, &[]);
        if let Some(uniform_group) = self.uniform_group.as_ref() {
            pass.set_bind_group(1, uniform_group, &[]);
        }

        pass.draw(0..3, 0..1);
    }
}
