pub extern crate wgpu;

pub mod defaults;
pub mod util;

mod texture;
pub use texture::{Texture, TextureView};

// Main instance
pub use wgpu::Instance;

// Backend selection
pub use wgpu::{Backend, Backends};

// Device
pub use wgpu::{util::DeviceExt, Adapter, Device, DeviceDescriptor};
// Device polling
pub use wgpu::Maintain;

pub use wgpu::{Features, Limits};

pub use wgpu::{PresentMode, Surface, SurfaceConfiguration};

// Pipeline State
pub use wgpu::{
    FragmentState, MultisampleState, RenderPipeline, RenderPipelineDescriptor, VertexState,
};
// Depth Stencil State
pub use wgpu::{DepthBiasState, DepthStencilState, StencilFaceState, StencilState};
// Primitive State
pub use wgpu::{Face, FrontFace, PolygonMode, PrimitiveState, PrimitiveTopology};

pub use wgpu::{RenderPassColorAttachment, RenderPassDepthStencilAttachment};

// Vertex Shader Buffers
pub use wgpu::{VertexAttribute, VertexBufferLayout, VertexStepMode};

// Fragment Shader Targets
pub use wgpu::{BlendComponent, BlendState, ColorTargetState, ColorWrites};

// Sampler
pub use wgpu::{AddressMode, FilterMode, Sampler, SamplerBorderColor, SamplerDescriptor};

pub use wgpu::{BindingResource, BindingType, BufferBinding, BufferBindingType};

pub use wgpu::{ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages};

pub use wgpu::{BufferUsages, TextureUsages};

pub use wgpu::{
    Texture as RawTexture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType,
};
pub use wgpu::{TextureView as RawTextureView, TextureViewDescriptor, TextureViewDimension};

pub use wgpu::{
    util::BufferInitDescriptor, BindGroup, BindGroupDescriptor, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BlendFactor, BlendOperation, Buffer,
    BufferAddress, BufferDescriptor, BufferSize, BufferView, BufferViewMut, Color, CommandBuffer,
    CommandBufferDescriptor, CommandEncoder, CommandEncoderDescriptor, CompareFunction,
    ComputePass, ComputePipeline, ComputePipelineDescriptor, DynamicOffset,
    Extent3d, /*Extensions,*/
    ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, IndexFormat, LoadOp, MapMode, Origin3d,
    PipelineLayout, PipelineLayoutDescriptor, PowerPreference, PushConstantRange, Queue,
    RenderPass, RenderPassDescriptor, RequestAdapterOptions, ShaderLocation, StencilOperation,
    VertexFormat, COPY_BUFFER_ALIGNMENT,
};
