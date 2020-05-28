use nannou::math::cgmath::{Matrix4, Rad, SquareMatrix, Vector3};
use nannou::wgpu;

use super::material::Material;
use super::Uniform;
use crate as lib;

pub type Vertex = ([f32; 3], [f32; 2], [f32; 3]);

pub struct VertexDescriptor;
impl wgpu::VertexDescriptor for VertexDescriptor {
    const STRIDE: wgpu::BufferAddress = std::mem::size_of::<Vertex>() as wgpu::BufferAddress;
    const ATTRIBUTES: &'static [wgpu::VertexAttributeDescriptor] = &[
        wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float3,
            offset: 0,
            shader_location: 0,
        },
        wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float2,
            offset: 3 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
            shader_location: 1,
        },
        wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float3,
            offset: 5 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
            shader_location: 2,
        },
    ];
}

pub struct Mesh {
    pub verts: Vec<Vertex>,
    pub buffer: wgpu::Buffer,
    size: wgpu::BufferAddress,
}

impl Mesh {
    pub fn new(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        verts: Vec<Vertex>,
    ) -> Self {
        let size = Self::size(verts.len());

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        let mesh = Self {
            verts,
            buffer,
            size,
        };

        mesh.update(device, encoder);

        mesh
    }

    fn size(n: usize) -> wgpu::BufferAddress {
        (n * std::mem::size_of::<Vertex>()) as wgpu::BufferAddress
    }

    pub fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        assert_eq!(
            self.size,
            Self::size(self.verts.len()),
            "mesh vertex buffer size changed!"
        );

        let staging = device
            .create_buffer_mapped(self.verts.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&self.verts);

        let size = (self.verts.len() * std::mem::size_of::<Vertex>()) as wgpu::BufferAddress;
        encoder.copy_buffer_to_buffer(&staging, 0, &self.buffer, 0, size);
    }
}

pub struct Transform {
    pub matrix: Matrix4<f32>,
    pub uniform: Uniform<TransformUniform>,
}

impl Transform {
    pub fn new(device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) -> Self {
        let transform = Self {
            matrix: Matrix4::identity(),
            uniform: Uniform::new(device),
        };

        transform
            .uniform
            .upload(device, encoder, transform.uniform());

        transform
    }

    pub fn translate(&mut self, pos: Vector3<f32>) {
        self.matrix = Matrix4::from_translation(pos);
    }

    pub fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        self.uniform.upload(device, encoder, self.uniform());
    }

    fn uniform(&self) -> TransformUniform {
        TransformUniform {
            matrix: self.matrix.clone(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TransformUniform {
    pub matrix: Matrix4<f32>,
}

pub struct Object {
    pub name: String,
    pub mesh: Mesh,
    pub material: Material,
    pub transform: Transform,
}

impl Object {
    pub fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        //self.mesh.update(device, encoder);
        self.material.update(device, encoder);
        self.transform.update(device, encoder);
    }
}

pub struct Model {
    pub objects: Vec<Object>,
    pub transform: Transform,
}

impl Model {
    pub fn new(
        device: &wgpu::Device,
        window: &nannou::window::Window,
        encoder: &mut wgpu::CommandEncoder,
        file: &str,
    ) -> Self {
        let data = lib::read_obj(file);

        let objects = data
            .objects
            .iter()
            .enumerate()
            .map(|(i, o)| {
                let mesh = Mesh::new(device, encoder, lib::wavefront::vertices(&data, o));
                let material = Material::new(device, window, lib::wavefront::material(&data, o));

                log::debug!("{} {}: {}", file, i, o.name);

                Object {
                    name: o.name.clone(),
                    mesh,
                    material,
                    transform: Transform::new(device, encoder),
                }
            })
            .collect();

        Self {
            objects,
            transform: Transform::new(device, encoder),
        }
    }

    pub fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        for o in &self.objects {
            o.update(device, encoder);
        }

        self.transform.update(device, encoder);
    }
}
