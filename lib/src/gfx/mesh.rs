use crate::gfx::uniform::Uniform;
use crate::gfx::wgpu::{self, VertexAttributeDescriptor, VertexFormat};
use crate::math::{Matrix4, Vector2, Vector3, Vector4};

pub enum VertexType {
    Simple,
    Skinned,
}

impl VertexType {
    pub fn attrs(&self) -> &'static [VertexAttributeDescriptor] {
        match self {
            VertexType::Simple => VERTEX_SIMPLE_ATTRS,
            VertexType::Skinned => VERTEX_SKINNED_ATTRS,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            VertexType::Simple => 8 * std::mem::size_of::<f32>(),
            VertexType::Skinned => 16 * std::mem::size_of::<f32>(),
        }
    }
}

const VERTEX_SIMPLE_ATTRS: &[VertexAttributeDescriptor] = &[
    VertexAttributeDescriptor {
        // pos
        offset: 0,
        format: VertexFormat::Float3,
        shader_location: 0,
    },
    VertexAttributeDescriptor {
        // tex
        offset: 3 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
        format: VertexFormat::Float2,
        shader_location: 1,
    },
    VertexAttributeDescriptor {
        // norm
        offset: 5 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
        format: VertexFormat::Float3,
        shader_location: 2,
    },
];

const VERTEX_SKINNED_ATTRS: &[VertexAttributeDescriptor] = &[
    VertexAttributeDescriptor {
        // pos
        offset: 0,
        format: VertexFormat::Float3,
        shader_location: 0,
    },
    VertexAttributeDescriptor {
        // tex
        offset: 3 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
        format: VertexFormat::Float2,
        shader_location: 1,
    },
    VertexAttributeDescriptor {
        // norm
        offset: 5 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
        format: VertexFormat::Float3,
        shader_location: 2,
    },
    VertexAttributeDescriptor {
        // skin indices
        offset: 8 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
        format: VertexFormat::Float4,
        shader_location: 3,
    },
    VertexAttributeDescriptor {
        // skin weights
        offset: 12 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
        format: VertexFormat::Float4,
        shader_location: 4,
    },
];

use safe_transmute::TriviallyTransmutable;
pub trait VertexExt: 'static + TriviallyTransmutable {
    fn ty() -> VertexType;
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub pos: Vector3,
    pub tex: Vector2,
    pub norm: Vector3,
}
unsafe impl TriviallyTransmutable for Vertex {}
impl VertexExt for Vertex {
    fn ty() -> VertexType {
        VertexType::Simple
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VertexSkinned {
    pub pos: Vector3,
    pub tex: Vector2,
    pub norm: Vector3,
    pub joints: Vector4,
    pub weights: Vector4,
}
unsafe impl TriviallyTransmutable for VertexSkinned {}
impl VertexExt for VertexSkinned {
    fn ty() -> VertexType {
        VertexType::Simple
    }
}

pub struct MeshDesc {
    pub ty: VertexType,
    pub verts: Vec<u8>,
    pub inds: Vec<u8>,
    pub n: u32,
}

impl MeshDesc {
    pub fn new<V: VertexExt>(verts: Vec<V>, inds: Vec<u32>) -> Self {
        let n = inds.len() as u32;

        let verts = safe_transmute::transmute_to_bytes(&verts).to_vec();
        let inds = safe_transmute::transmute_to_bytes(&inds).to_vec();

        Self {
            ty: V::ty(),
            verts,
            inds,
            n,
        }
    }
}

pub struct Mesh {
    pub group: wgpu::BindGroup,

    pub transform: Uniform<Matrix4>,

    pub vertex: wgpu::Buffer,
    pub index: wgpu::Buffer,
    pub n: u32,
}

impl Mesh {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        desc: &MeshDesc,
        transform: &Matrix4,
    ) -> Self {
        let vertex = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertices"),
            usage: wgpu::BufferUsage::VERTEX,
            size: desc.verts.len() as wgpu::BufferAddress,
            mapped_at_creation: true,
        });

        vertex
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(&desc.verts);

        vertex.unmap();

        let index = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("indices"),
            usage: wgpu::BufferUsage::INDEX,
            size: desc.inds.len() as wgpu::BufferAddress,
            mapped_at_creation: true,
        });

        index
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(&desc.inds);

        index.unmap();

        let transform = Uniform::new(device, "model_transform", Some(transform));

        let group = wgpu::util::BindGroupBuilder::new("model")
            .uniform(&transform)
            .build(device, layout);

        Self {
            group,

            transform,

            vertex,
            index,
            n: desc.n,
        }
    }

    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, group_idx: u32) {
        pass.set_bind_group(group_idx, &self.group, &[]);

        pass.set_vertex_buffer(0, self.vertex.slice(..));
        pass.set_index_buffer(self.index.slice(..));

        pass.draw_indexed(0..self.n, 0, 0..1);
    }

    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        wgpu::util::BindGroupLayoutBuilder::new("model")
            .uniform(wgpu::ShaderStage::VERTEX)
            .build(device)
    }
}
