use nannou::wgpu;
use nannou::math::cgmath::{self, Deg, Matrix4, Point3, Vector3};

use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;

// TODO: put this shit in multiple files

pub const BILLBOARD_SHADER: &'static str = "../resources/shaders/billboard.vert.spv";
pub const PASSTHROUGH_SHADER: &'static str = "../resources/shaders/passthrough.frag.spv";

pub const TEXTURE_SIZE: [u32; 2] = [1920, 1080];
pub const TEXTURE_ASPECT: f32 = 1920.0 / 1080.0;
pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub fn texture_builder() -> wgpu::TextureBuilder {
    wgpu::TextureBuilder::new()
        .size(TEXTURE_SIZE)
        .format(TEXTURE_FORMAT)
}

pub fn depth_builder() -> wgpu::TextureBuilder {
    wgpu::TextureBuilder::new()
        .size(TEXTURE_SIZE)
        .format(DEPTH_FORMAT)
        .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT)
}

pub struct Effect<T: Debug + Copy + Clone + 'static = ()> {
    texture_view: wgpu::TextureView,
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

        let texture_view = texture.view().build();

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
            .texture_view(&texture_view)
            .sampler(&sampler);
        if let Some(buffer) = &uniform {
            group_builder = group_builder.buffer::<T>(&buffer, 0..1);
        }
        let bind_group = group_builder.build(device, &bind_group_layout);

        let pipeline_layout = wgpu::create_pipeline_layout(device, &[&bind_group_layout]);
        let pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
            .fragment_shader(&fs_mod)
            .color_format(TEXTURE_FORMAT)
            .sample_count(samples)
            .build(device);

        Self {
            texture_view,
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
        &self.texture_view
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

        let reshaper =
            wgpu::TextureReshaper::new(device, &texture_view, samples, 1, TEXTURE_FORMAT);

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

pub struct Uniform<T: Copy + Clone + 'static> {
    buffer: wgpu::Buffer,
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

    pub fn upload(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, t: T) {
        let staging = device
            .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(std::slice::from_ref(&t));

        encoder.copy_buffer_to_buffer(&staging, 0, &self.buffer, 0, Self::SIZE);
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

pub struct Mesh {
    pub buffer: wgpu::Buffer,
    pub texture: Option<wgpu::TextureView>,
    pub len: u32,
}

impl Mesh {
    pub fn new(
        device: &wgpu::Device,
        window: &nannou::window::Window,
        encoder: &mut wgpu::CommandEncoder,
        mesh: &super::MeshData,
    ) -> Self {
        let size = (mesh.data.len() * std::mem::size_of::<super::VertTexNorm>()) as wgpu::BufferAddress;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        let staging = device
            .create_buffer_mapped(mesh.data.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&mesh.data);

        encoder.copy_buffer_to_buffer(&staging, 0, &buffer, 0, size);

        let texture = mesh.material.uv_map.as_ref().map(|file| {
            let image = nannou::image::open(super::resource(file)).expect(&format!("{} not found", file));
            let texture = wgpu::Texture::from_image(window, &image);
            texture.view().build()
        });

        Self {
            buffer,
            texture,
            len: mesh.data.len() as u32,
        }
    }
}

// TODO: Have a Uniform trait that we can derive to
// auto generate thing that returns self.clone
//, OR we can overload it (with CameraDesc -> CameraUniform)
// to transform data before upload

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CameraUniform {
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

pub struct CameraDesc {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

pub struct Camera {
    pub desc: CameraDesc,
    uniform: Uniform<CameraUniform>,
}

impl Camera {
    pub fn new(device: &wgpu::Device, desc: CameraDesc) -> Self {
        Self {
            desc,
            uniform: Uniform::new(device)
        }
    }

    pub fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        self.uniform.upload(device, encoder, self.uniform());
    }

    fn uniform(&self) -> CameraUniform {
        let d = &self.desc;
        let view = Matrix4::look_at(d.eye, d.target, d.up);
        let proj = cgmath::perspective(
            Deg(d.fov / TEXTURE_ASPECT),
            TEXTURE_ASPECT,
            d.near,
            d.far,
        );

        CameraUniform { view, proj }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.uniform.buffer
    }
}