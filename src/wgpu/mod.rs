// TODO: Remove. Only needed to complete with rust-analyzer for some reason?
extern crate wgpu;

pub mod defaults;

pub mod frame;

pub use wgpu::{
    Adapter, AdapterInfo, AddressMode, Backend, BackendBit, BindGroup, BindGroupDescriptor,
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    /*Binding,*/ BindingResource, BindingType, BlendDescriptor, BlendFactor, BlendOperation,
    Buffer, BufferAddress, /*BufferAsyncErr,*/ BufferCopyView, BufferDescriptor,
    BufferInitDescriptor, /*BufferReadMapping,*/ BufferUsage,
    /*BufferWriteMapping,*/ Color, ColorStateDescriptor, ColorWrite, CommandBuffer,
    CommandBufferDescriptor, CommandEncoder, CommandEncoderDescriptor, CompareFunction,
    ComputePass, ComputePipeline, ComputePipelineDescriptor, /*CreateBufferMapped,*/ CullMode,
    DepthStencilStateDescriptor, Device, DeviceDescriptor, DeviceExt, DeviceType, DynamicOffset,
    Extent3d, Features, /*Extensions,*/
    FilterMode, FrontFace, IndexFormat, InputStepMode, Instance, Limits, LoadOp, Maintain,
    Origin3d, PipelineLayout, PipelineLayoutDescriptor, PowerPreference, PresentMode,
    PrimitiveTopology, ProgrammableStageDescriptor, Queue, RasterizationStateDescriptor,
    RenderPass, RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler,
    SamplerDescriptor, ShaderLocation, ShaderModule, ShaderStage, StencilOperation,
    StencilStateFaceDescriptor, /*StoreOp,*/ Surface, SwapChain, SwapChainDescriptor,
    SwapChainFrame, /*SwapChainOutput,*/
    Texture,        /*as TextureHandle*/
    TextureAspect, TextureComponentType, TextureCopyView, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsage, TextureView, /*as TextureViewHandle*/
    TextureViewDescriptor, TextureViewDimension, /*TimeOut,*/
    VertexAttributeDescriptor, VertexBufferDescriptor, VertexFormat, VertexStateDescriptor,
    BIND_BUFFER_ALIGNMENT, /*MAX_BIND_GROUPS,*/
};
