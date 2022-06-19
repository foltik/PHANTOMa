use crate::gfx::wgpu;
use crate::gfx::wgpu::{Device, PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModule};

use super::PipelineBuilder;

// FIXME: ADD BUILDERS BACK FOR EVERYTHING!
// FIXME: STOP ASSUMING A SINGLE COLOR STATE!

// //! Items aimed at easing the contruction of a render pipeline.
// //!
// //! Creating a `RenderPipeline` tends to involve a lot of boilerplate that we don't always want to
// //! have to consider when writing graphics code. Here we define a set of helpers that allow us to
// //! simplify the process and fall back to a set of reasonable defaults.

/// A builder type to help simplify the construction of a **RenderPipeline**.
///
/// We've attempted to provide a suite of reasonable defaults in the case that none are provided.
#[derive(Debug)]
pub struct RenderPipelineBuilder<'a> {
    builder: PipelineBuilder<'a>,

    vs_mod: &'a wgpu::ShaderModule,
    vs_buffers: Vec<wgpu::VertexBufferLayout<'a>>,

    fs_mod: Option<&'a wgpu::ShaderModule>,
    fs_targets: Vec<wgpu::ColorTargetState>,

    primitive: wgpu::PrimitiveState,
    multisample: wgpu::MultisampleState,

    depth_stencil: Option<wgpu::DepthStencilState>,
}

impl<'a> RenderPipelineBuilder<'a> {
    // The default entry point used for shaders when unspecified.
    pub const DEFAULT_SHADER_ENTRY_POINT: &'static str = "main";

    pub const DEFAULT_PRIMITIVE_STATE: wgpu::PrimitiveState = wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: None,
        clamp_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        conservative: false,
    };

    pub const DEFAULT_DEPTH_STENCIL_STATE: wgpu::DepthStencilState = wgpu::DepthStencilState {
        format: wgpu::defaults::depth_format(),
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::LessEqual,
        bias: wgpu::DepthBiasState {
            constant: 0,
            slope_scale: 0.0,
            clamp: 0.0,
        },
        stencil: wgpu::StencilState {
            front: wgpu::StencilFaceState::IGNORE,
            back: wgpu::StencilFaceState::IGNORE,
            read_mask: 0,
            write_mask: 0,
        }
    };

    pub const DEFAULT_MULTISAMPLE_STATE: wgpu::MultisampleState = wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
    };

    // Color state defaults.
    pub const DEFAULT_COLOR_WRITE: wgpu::ColorWrites = wgpu::ColorWrites::ALL;
    pub const DEFAULT_COLOR_BLEND: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::SrcAlpha,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    pub const DEFAULT_ALPHA_BLEND: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    pub const DEFAULT_BLEND_STATE: wgpu::BlendState = wgpu::BlendState {
        color: Self::DEFAULT_COLOR_BLEND,
        alpha: Self::DEFAULT_ALPHA_BLEND,
    };
    pub const DEFAULT_COLOR_TARGET_STATE: wgpu::ColorTargetState = wgpu::ColorTargetState {
        format: wgpu::defaults::texture_format(),
        blend: None,
        write_mask: Self::DEFAULT_COLOR_WRITE,
    };

    // Constructors

    pub fn new(builder: PipelineBuilder<'a>, vs_mod: &'a ShaderModule) -> Self {
        Self {
            builder,
            
            vs_mod,
            vs_buffers: Vec::new(),

            fs_mod: None,
            fs_targets: Vec::new(),

            depth_stencil: None,

            multisample: Self::DEFAULT_MULTISAMPLE_STATE,
            primitive: Self::DEFAULT_PRIMITIVE_STATE,
        }
    }

    // Builders

    /// Specify a fragment shader for the render pipeline.
    pub fn fragment(mut self, module: &'a wgpu::ShaderModule) -> Self {
        self.fs_mod = Some(module);
        self.fs_targets.push(Self::DEFAULT_COLOR_TARGET_STATE);
        self
    }

    // Rasterization state.

    /// Specify the full rasterization state.
    // pub fn rasterization_state(mut self, state: wgpu::RasterizationStateDescriptor) -> Self {
    //     self.rasterization_state = Some(state);
    //     self
    // }

    // pub fn front_face(mut self, front_face: wgpu::FrontFace) -> Self {
    //     let state = self
    //         .rasterization_state
    //         .get_or_insert(Self::DEFAULT_RASTERIZATION_STATE);
    //     state.front_face = front_face;
    //     self
    // }

    // pub fn cull_mode(mut self, cull_mode: wgpu::CullMode) -> Self {
    //     let state = self
    //         .rasterization_state
    //         .get_or_insert(Self::DEFAULT_RASTERIZATION_STATE);
    //     state.cull_mode = cull_mode;
    //     self
    // }

    // pub fn depth_bias(mut self, bias: i32) -> Self {
    //     let state = self
    //         .rasterization_state
    //         .get_or_insert(Self::DEFAULT_RASTERIZATION_STATE);
    //     state.depth_bias = bias;
    //     self
    // }

    // pub fn depth_bias_slope_scale(mut self, scale: f32) -> Self {
    //     let state = self
    //         .rasterization_state
    //         .get_or_insert(Self::DEFAULT_RASTERIZATION_STATE);
    //     state.depth_bias_slope_scale = scale;
    //     self
    // }

    // pub fn depth_bias_clamp(mut self, clamp: f32) -> Self {
    //     let state = self
    //         .rasterization_state
    //         .get_or_insert(Self::DEFAULT_RASTERIZATION_STATE);
    //     state.depth_bias_clamp = clamp;
    //     self
    // }

    // Primitive topology.

    /// Specify the primitive topology.
    ///
    /// This represents the way vertices will be read from the **VertexBuffer**.
    // pub fn primitive_topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
    //     self.primitive_topology = topology;
    //     self
    // }

    // Color state.

    /// Specify the full color state for drawing to the output attachment.
    ///
    /// If you have multiple output attachments, see the `color_states` method.
    // pub fn color_state(mut self, state: wgpu::ColorStateDescriptor) -> Self {
    //     self.color_state = Some(state);
    //     self
    // }

    // pub fn color_format(mut self, format: wgpu::TextureFormat) -> Self {
    //     let state = self.color_state.get_or_insert(Self::DEFAULT_COLOR_STATE);
    //     state.format = format;
    //     self
    // }

    pub fn color_blend(mut self, blend: wgpu::BlendComponent) -> Self {
        let target = &mut self.fs_targets[0];
        let mut state = target.blend.unwrap_or(Self::DEFAULT_BLEND_STATE);
        state.color = blend;
        target.blend = Some(state);
        self
    }

    pub fn alpha_blend(mut self, blend: wgpu::BlendComponent) -> Self {
        let target = &mut self.fs_targets[0];
        let mut state = target.blend.unwrap_or(Self::DEFAULT_BLEND_STATE);
        state.alpha = blend;
        target.blend = Some(state);
        self
    }

    pub fn write_mask(mut self, mask: wgpu::ColorWrites) -> Self {
        let target = &mut self.fs_targets[0];
        target.write_mask = mask;
        self
    }

    // // TODO: rethink this instead of just having this hack
    // pub fn color_states<const N: usize>(mut self) -> Self {
    //     self.color_states = &[Self::DEFAULT_COLOR_STATE; N];
    //     self
    // }

    // Depth / Stencil state

    // pub fn depth_stencil(mut self) -> Self {
    //     self.depth_stencil = Some(Self::DEFAULT_DEPTH_STENCIL_STATE);
    //     self
    // }

    // pub fn depth_stencil_state(mut self, state: wgpu::DepthStencilStateDescriptor) -> Self {
    //     self.depth_stencil = Some(state);
    //     self
    // }

    // pub fn depth_format(mut self, format: wgpu::TextureFormat) -> Self {
    //     let state = self
    //         .depth_stencil
    //         .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
    //     state.format = format;
    //     self
    // }

    // pub fn depth_write_enabled(mut self, enabled: bool) -> Self {
    //     let state = self
    //         .depth_stencil
    //         .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
    //     state.depth_write_enabled = enabled;
    //     self
    // }

    // pub fn depth_compare(mut self, compare: wgpu::CompareFunction) -> Self {
    //     let state = self
    //         .depth_stencil
    //         .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
    //     state.depth_compare = compare;
    //     self
    // }

    // pub fn stencil_front(mut self, stencil: wgpu::StencilStateFaceDescriptor) -> Self {
    //     let state = self
    //         .depth_stencil
    //         .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
    //     state.stencil.front = stencil;
    //     self
    // }

    // pub fn stencil_back(mut self, stencil: wgpu::StencilStateFaceDescriptor) -> Self {
    //     let state = self
    //         .depth_stencil
    //         .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
    //     state.stencil.back = stencil;
    //     self
    // }

    // pub fn stencil_read_mask(mut self, mask: u32) -> Self {
    //     let state = self
    //         .depth_stencil
    //         .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
    //     state.stencil.read_mask = mask;
    //     self
    // }

    // pub fn stencil_write_mask(mut self, mask: u32) -> Self {
    //     let state = self
    //         .depth_stencil
    //         .get_or_insert(Self::DEFAULT_DEPTH_STENCIL_STATE);
    //     state.stencil.write_mask = mask;
    //     self
    // }

    // Vertex buffer methods.

    /// The format of the type used within the index buffer.
    // pub fn index_format(mut self, format: wgpu::IndexFormat) -> Self {
    //     self.index_format = format;
    //     self
    // }

    // FIXME: ADD THE BUILDERS ALREADY!
    pub fn hack_add_default_depth_stencil_state(mut self) -> Self {
        self.depth_stencil = Some(Self::DEFAULT_DEPTH_STENCIL_STATE);
        // self.primitive.strip_index_format = Some(wgpu::IndexFormat::Uint32);
        self
    }

    /// Add a new vertex buffer layout to the vertex state.
    pub fn add_vertex_buffer_layout(
        mut self,
        layout: wgpu::VertexBufferLayout<'static>,
    ) -> Self {
        self.vs_buffers.push(layout);
        self
    }

    /// Short-hand for adding a layout to the render pipeline describing a buffer of vertices
    /// of the given vertex type.
    ///
    /// The vertex stride is assumed to be equal to `size_of::<V>()`. If this is not the case,
    /// consider using `add_vertex_buffer_descriptor` instead.
    pub fn add_vertex_buffer<V>(self, attrs: &'static [wgpu::VertexAttribute]) -> Self {
        let array_stride = std::mem::size_of::<V>() as wgpu::BufferAddress;
        let step_mode = wgpu::VertexStepMode::Vertex;
        self.add_vertex_buffer_layout(wgpu::VertexBufferLayout {
            array_stride,
            step_mode,
            attributes: attrs,
        })
    }

    /// Short-hand for adding a layout to the render pipeline describing a buffer of instances
    /// of the given vertex type.
    pub fn add_instance_buffer<I>(self, attrs: &'static [wgpu::VertexAttribute]) -> Self {
        let array_stride = std::mem::size_of::<I>() as wgpu::BufferAddress;
        let step_mode = wgpu::VertexStepMode::Instance;
        self.add_vertex_buffer_layout(wgpu::VertexBufferLayout {
            array_stride,
            step_mode,
            attributes: attrs,
        })
    }

    /// The sample count of the output attachment.
    // pub fn sample_count(mut self, sample_count: u32) -> Self {
    //     self.sample_count = sample_count;
    //     self
    // }

    // Finalising methods.

    /// Build the render pipeline layout, its descriptor and ultimately the pipeline itself with
    /// the specified parameters.
    ///
    /// **Panic!**s in the following occur:
    ///
    /// - A rasterization state field was specified but no fragment shader was given.
    /// - A color state field was specified but no fragment shader was given.
    pub fn build(self, device: &Device) -> RenderPipeline {
        let Self {
            builder,
            
            vs_mod,
            vs_buffers,

            fs_mod,
            fs_targets,

            depth_stencil,

            primitive,
            multisample
        } = self;

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("{}_pipeline_layout", builder.label)),
            bind_group_layouts: &builder.layouts,
            push_constant_ranges: &builder.constants
        });

        let label = &format!("{}_pipeline", builder.label);

        let pipeline_desc = RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&layout),

            vertex: wgpu::VertexState {
                module: vs_mod,
                entry_point: "main",
                buffers: &vs_buffers,
            },
            fragment: fs_mod.map(|fs_mod| wgpu::FragmentState {
                module: fs_mod,
                entry_point: "main",
                targets: &fs_targets,
            }),

            depth_stencil,

            primitive,
            multisample,
        };

        device.create_render_pipeline(&pipeline_desc)
    }
}
