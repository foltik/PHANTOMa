use nannou::wgpu;

use crate as lib;

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
        mesh: &lib::MeshData,
    ) -> Self {
        let size = (mesh.data.len() * std::mem::size_of::<crate::VertTexNorm>()) as wgpu::BufferAddress;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        let staging = device
            .create_buffer_mapped(mesh.data.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&mesh.data);

        encoder.copy_buffer_to_buffer(&staging, 0, &buffer, 0, size);

        let texture = mesh.material.uv_map.as_ref().map(|file| {
            log::debug!("Loading {} for {}", file, mesh.name);
            let image = nannou::image::open(lib::resource(file)).expect(&format!("{} not found", file));
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