pub mod camera;
pub mod lights;
pub mod material;
pub mod mesh;
pub mod model;
pub mod scene;

use crate as lib;

use nannou::wgpu;

use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;

pub use camera::{Camera, CameraDesc, CameraUniform};
pub use mesh::Mesh;

// TODO: put this shit in multiple files

pub const BILLBOARD_SHADER: &'static str = "../resources/shaders/billboard.vert.spv";
pub const PASSTHROUGH_SHADER: &'static str = "../resources/shaders/passthrough.frag.spv";

pub const RESOLUTION: [u32; 2] = [1920, 1080];
pub const ASPECT: f32 = 1920.0 / 1080.0;
pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub fn texture_builder() -> wgpu::TextureBuilder {
    wgpu::TextureBuilder::new().size(RESOLUTION).format(FORMAT)
}

pub fn depth_builder() -> wgpu::TextureBuilder {
    wgpu::TextureBuilder::new()
        .size(RESOLUTION)
        .format(DEPTH_FORMAT)
        .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT)
}

pub struct Effect<T: Debug + Copy + Clone + 'static = ()> {
    pub view: wgpu::TextureView,
    uniform: Option<wgpu::Buffer>,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    phantom: PhantomData<T>,
}

impl<T: Debug + Copy + Clone + 'static> Effect<T> {
    const SIZE: wgpu::BufferAddress = std::mem::size_of::<T>() as wgpu::BufferAddress;

    fn new_internal(device: &wgpu::Device, samples: u32, fragment: &str) -> Self {
        let shader = |p| wgpu::shader_from_spirv_bytes(device, &super::read_resource_raw(p));
        let vs_mod = shader(BILLBOARD_SHADER);
        let fs_mod = shader(fragment);

        let texture = texture_builder()
            .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
            .build(device);

        let view = texture.view().build();

        let uniform = if Self::SIZE != 0 {
            Some(device.create_buffer(&wgpu::BufferDescriptor {
                size: Self::SIZE,
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }))
        } else {
            None
        };

        let sampler = wgpu::SamplerBuilder::new().build(&device);

        // TODO uniform should be in its own bind group
        let mut layout_builder = wgpu::BindGroupLayoutBuilder::new()
            .sampled_texture_from(wgpu::ShaderStage::FRAGMENT, &texture)
            .sampler(wgpu::ShaderStage::FRAGMENT);
        if let Some(_) = &uniform {
            layout_builder = layout_builder.uniform_buffer(wgpu::ShaderStage::FRAGMENT, false);
        }
        let bind_group_layout = layout_builder.build(device);

        let mut group_builder = wgpu::BindGroupBuilder::new()
            .texture_view(&view)
            .sampler(&sampler);
        if let Some(buffer) = &uniform {
            group_builder = group_builder.buffer::<T>(&buffer, 0..1);
        }
        let bind_group = group_builder.build(device, &bind_group_layout);

        let pipeline_layout = wgpu::create_pipeline_layout(device, &[&bind_group_layout]);
        let pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
            .fragment_shader(&fs_mod)
            .color_format(FORMAT)
            .sample_count(samples)
            .build(device);

        Self {
            view,
            uniform,
            bind_group,
            pipeline,
            phantom: PhantomData,
        }
    }

    pub fn new(device: &wgpu::Device, fragment: &str) -> Self {
        Self::new_internal(device, 1, fragment)
    }

    pub fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, uniform: &T) {
        if let Some(buffer) = &self.uniform {
            let staging = device
                .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
                .fill_from_slice(std::slice::from_ref(uniform));

            encoder.copy_buffer_to_buffer(&staging, 0, buffer, 0, Self::SIZE);
        }
    }

    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let mut pass = wgpu::RenderPassBuilder::new()
            .color_attachment(target, |c| c)
            .begin(encoder);

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);

        pass.draw(0..3, 0..1);
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

// TODO: this is mostly an Effect with two input images and a hard coded
// shader that sums the two image values.
// I only made it because for some reason a Draw clears out the image from Maze
// before drawing, so either figure out how to genericize Effect over N input
// images, or fix Draw clearing the image.
pub struct Composite {
    pub view1: wgpu::TextureView,
    pub view2: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl Composite {
    pub fn new(device: &wgpu::Device) -> Self {
        let vs_mod = lib::read_shader(device, BILLBOARD_SHADER);
        let fs_mod = lib::read_shader(device, "add.frag.spv");

        let tex1 = texture_builder()
            .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
            .build(device);
        let view1 = tex1.view().build();

        let tex2 = texture_builder()
            .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
            .build(device);
        let view2 = tex2.view().build();

        let sampler = wgpu::SamplerBuilder::new().build(&device);

        let bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
            .sampled_texture_from(wgpu::ShaderStage::FRAGMENT, &tex1)
            .sampled_texture_from(wgpu::ShaderStage::FRAGMENT, &tex2)
            .sampler(wgpu::ShaderStage::FRAGMENT)
            .build(device);

        let bind_group = wgpu::BindGroupBuilder::new()
            .texture_view(&view1)
            .texture_view(&view2)
            .sampler(&sampler)
            .build(device, &bind_group_layout);

        let pipeline_layout = wgpu::create_pipeline_layout(device, &[&bind_group_layout]);
        let pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
            .fragment_shader(&fs_mod)
            .color_format(FORMAT)
            .build(device);

        Self {
            view1,
            view2,
            bind_group,
            pipeline,
        }
    }

    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let mut pass = wgpu::RenderPassBuilder::new()
            .color_attachment(target, |c| c)
            .begin(encoder);

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);

        pass.draw(0..3, 0..1);
    }
}

pub struct Present {
    effect: Effect<()>,
}

impl Present {
    pub fn new(device: &wgpu::Device, samples: u32) -> Self {
        Self {
            effect: Effect::new_internal(device, samples, PASSTHROUGH_SHADER),
        }
    }

    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, frame: &nannou::Frame) {
        self.effect.encode(encoder, frame.texture_view())
    }

    pub fn view(&self) -> &wgpu::TextureView {
        self.effect.view()
    }
}

pub struct Drawer {
    renderer: RefCell<nannou::draw::Renderer>,
    reshaper: wgpu::TextureReshaper,
    texture: wgpu::Texture,
}

impl Drawer {
    pub fn new(device: &wgpu::Device, samples: u32) -> Self {
        let texture = texture_builder()
            .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
            .sample_count(samples)
            .build(device);

        let texture_view = texture.view().build();

        let renderer = nannou::draw::RendererBuilder::new()
            .build_from_texture_descriptor(device, texture.descriptor());

        let reshaper = wgpu::TextureReshaper::new(device, &texture_view, samples, 1, FORMAT);

        Self {
            renderer: RefCell::new(renderer),
            reshaper,
            texture,
        }
    }

    pub fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        draw: &nannou::Draw,
    ) {
        self.renderer
            .borrow_mut()
            .render_to_texture(device, encoder, draw, &self.texture);

        self.reshaper.encode_render_pass(target, encoder);
    }
}

/*
pub trait UniformData {
    type Data;
    fn uniform(&self) -> Self::Data;
}
*/

pub struct Uniform<T: Copy + Clone + 'static> {
    pub buffer: wgpu::Buffer,
    data: PhantomData<T>,
}

impl<T: Copy + Clone + 'static> Uniform<T> {
    const SIZE: wgpu::BufferAddress = std::mem::size_of::<T>() as wgpu::BufferAddress;

    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: Self::SIZE,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        Self {
            buffer,
            data: PhantomData,
        }
    }

    pub fn new_array(device: &wgpu::Device, n: usize) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: Self::SIZE * n as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        Self {
            buffer,
            data: PhantomData,
        }
    }

    pub fn upload(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, t: T) {
        let staging = device
            .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(std::slice::from_ref(&t));

        encoder.copy_buffer_to_buffer(&staging, 0, &self.buffer, 0, Self::SIZE);
    }

    pub fn upload_slice(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        ts: &[T],
    ) {
        let n = ts.len();
        let staging = device
            .create_buffer_mapped(n, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(ts);

        encoder.copy_buffer_to_buffer(
            &staging,
            0,
            &self.buffer,
            0,
            Self::SIZE * n as wgpu::BufferAddress,
        );
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

pub struct UniformStorage<T: Copy + Clone + 'static> {
    pub v: T,
    uniform: Uniform<T>,
}

impl<T: Copy + Clone + 'static> UniformStorage<T> {
    pub fn new(device: &wgpu::Device, v: T) -> Self {
        Self {
            v,
            uniform: Uniform::new(device),
        }
    }

    pub fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        self.uniform.upload(device, encoder, self.v.clone());
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        self.uniform.buffer()
    }
}
