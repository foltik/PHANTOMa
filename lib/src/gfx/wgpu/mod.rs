pub extern crate wgpu;

pub mod defaults;
pub mod util;

mod texture;
pub use texture::{Texture, TextureView, SwapChainTextureView};


// Main instance
pub use wgpu::Instance;

// Backend selection
pub use wgpu::{/*Backend, */BackendBit};

// Device
pub use wgpu::{Adapter, Device, DeviceDescriptor};
// Device polling
pub use wgpu::Maintain;

pub use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor,
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    /*Binding,*/ BindingResource, BindingType, BlendDescriptor, BlendFactor, BlendOperation,
    Buffer, BufferAddress, /*BufferAsyncErr,*/ BufferCopyView, BufferDescriptor,
    /*BufferReadMapping,*/ BufferUsage, /*BufferWriteMapping,*/ Color,
    ColorStateDescriptor, ColorWrite, CommandBuffer, CommandBufferDescriptor, CommandEncoder,
    CommandEncoderDescriptor, CompareFunction, ComputePass, ComputePipeline,
    ComputePipelineDescriptor, /*CreateBufferMapped,*/ CullMode, DepthStencilStateDescriptor,
    DynamicOffset, Extent3d, Features, /*Extensions,*/
    FilterMode, FrontFace, IndexFormat, InputStepMode, Limits, LoadOp,
    Origin3d, PipelineLayout, PipelineLayoutDescriptor, PowerPreference, PresentMode,
    PrimitiveTopology, ProgrammableStageDescriptor, PushConstantRange, Queue, RasterizationStateDescriptor,
    RenderPass, RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler,
    SamplerDescriptor, ShaderLocation, ShaderModule, ShaderStage, StencilOperation,
    StencilStateFaceDescriptor, StencilStateDescriptor, /*StoreOp,*/ Surface, SwapChain,
    SwapChainDescriptor, SwapChainFrame, /*SwapChainOutput,*/
    Texture as RawTexture,
    TextureAspect, TextureComponentType, TextureCopyView, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsage, TextureView as RawTextureView,
    TextureViewDescriptor, TextureViewDimension, /*TimeOut,*/
    VertexAttributeDescriptor, VertexBufferDescriptor, VertexFormat, VertexStateDescriptor,
    BIND_BUFFER_ALIGNMENT, /*MAX_BIND_GROUPS,*/
};
