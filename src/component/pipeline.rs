#![allow(dead_code)]
#![allow(unused_imports)]

use rendy::{
    factory::Factory,
    graph::render::Layout,
    hal::{
        device::Device,
        pass::Subpass,
        pso::{
            AttributeDesc, BakedStates, BasePipeline, BlendDesc, ColorBlendDesc, DepthStencilDesc,
            DepthTest, Face, GraphicsPipelineDesc, GraphicsShaderSet, InputAssemblerDesc,
            Multisampling, PipelineCreationFlags, Primitive, Rasterizer, Rect, VertexBufferDesc,
            VertexInputRate, Viewport,
        },
        Backend,
    },
    shader::{SpirvReflection},
    mesh::VertexFormat,
};

#[derive(Debug)]
enum BasePipelineExt<'a, P> {
    Pipeline(&'a P),
    Index(usize),
    None,
}

#[derive(Debug)]
pub struct PipelineDescBuilder<'a, B: Backend> {
    reflect: Option<SpirvReflection>,
    shaders: Option<GraphicsShaderSet<'a, B>>,
    rasterizer: Rasterizer,
    vertex_buffers: Vec<VertexBufferDesc>,
    attributes: Vec<AttributeDesc>,
    input_assembler: InputAssemblerDesc,
    blender: BlendDesc,
    depth_stencil: DepthStencilDesc,
    multisampling: Option<Multisampling>,
    baked_states: BakedStates,
    layout: Option<&'a B::PipelineLayout>,
    subpass: Option<Subpass<'a, B>>,
    flags: PipelineCreationFlags,
    parent: BasePipelineExt<'a, B::GraphicsPipeline>,
}

impl<'a, B: Backend> Default for PipelineDescBuilder<'a, B> {
    fn default() -> Self {
        Self {
            reflect: None,
            shaders: None,
            rasterizer: Rasterizer::FILL,
            vertex_buffers: Vec::new(),
            attributes: Vec::new(),
            input_assembler: InputAssemblerDesc::new(Primitive::TriangleList),
            blender: BlendDesc::default(),
            depth_stencil: DepthStencilDesc::default(),
            multisampling: None,
            baked_states: BakedStates::default(),
            layout: None,
            subpass: None,
            flags: PipelineCreationFlags::empty(),
            parent: BasePipelineExt::None,
        }
    }
}

impl<'a, B: Backend> PipelineDescBuilder<'a, B> {
    pub fn build(mut self, factory: &Factory<B>) -> GraphicsPipelineDesc<'a, B> {
        GraphicsPipelineDesc {
            shaders: self.shaders.expect("No shaders specified for pipeline"),
            rasterizer: self.rasterizer,
            vertex_buffers: self.vertex_buffers,
            attributes: self.attributes,
            input_assembler: self.input_assembler,
            blender: self.blender,
            depth_stencil: self.depth_stencil,
            multisampling: self.multisampling,
            baked_states: self.baked_states,
            layout: self.layout.expect("No layout specified for pipeline"),
            subpass: self.subpass.expect("No subpass specified for pipeline"),
            flags: self.flags,
            parent: match self.parent {
                BasePipelineExt::Pipeline(p) => BasePipeline::Pipeline(p),
                BasePipelineExt::Index(i) => BasePipeline::Index(i),
                BasePipelineExt::None => BasePipeline::None,
            },
        }
    }

    pub fn set_shaders(&mut self, shaders: GraphicsShaderSet<'a, B>) {
        self.shaders.replace(shaders);
    }
    pub fn with_shaders(mut self, shaders: GraphicsShaderSet<'a, B>) -> Self {
        self.set_shaders(shaders);
        self
    }

    pub fn set_rasterizer(&mut self, rasterizer: Rasterizer) {
        self.rasterizer = rasterizer;
    }
    pub fn with_rasterizer(mut self, rasterizer: Rasterizer) -> Self {
        self.set_rasterizer(rasterizer);
        self
    }

    pub fn set_vertex_buffers(&mut self, vertex_buffers: Vec<VertexBufferDesc>) {
        self.vertex_buffers = vertex_buffers;
    }
    pub fn with_vertex_buffers(mut self, vertex_buffers: Vec<VertexBufferDesc>) -> Self {
        self.set_vertex_buffers(vertex_buffers);
        self
    }

    pub fn set_attributes(&mut self, attributes: Vec<AttributeDesc>) {
        self.attributes = attributes;
    }
    pub fn with_attributes(mut self, attributes: Vec<AttributeDesc>) -> Self {
        self.set_attributes(attributes);
        self
    }

    pub fn set_input_assembler(&mut self, input_assembler: InputAssemblerDesc) {
        self.input_assembler = input_assembler;
    }
    pub fn with_input_assembler(mut self, input_assembler: InputAssemblerDesc) -> Self {
        self.set_input_assembler(input_assembler);
        self
    }

    pub fn set_blender(&mut self, blender: BlendDesc) {
        self.blender = blender;
    }
    pub fn with_blender(mut self, blender: BlendDesc) -> Self {
        self.set_blender(blender);
        self
    }

    pub fn set_depth_stencil(&mut self, depth_stencil: DepthStencilDesc) {
        self.depth_stencil = depth_stencil;
    }
    pub fn with_depth_stencil(mut self, depth_stencil: DepthStencilDesc) -> Self {
        self.set_depth_stencil(depth_stencil);
        self
    }

    pub fn set_multisampling(&mut self, multisampling: Option<Multisampling>) {
        self.multisampling = multisampling;
    }
    pub fn with_multisampling(mut self, multisampling: Option<Multisampling>) -> Self {
        self.set_multisampling(multisampling);
        self
    }

    pub fn set_baked_states(&mut self, baked_states: BakedStates) {
        self.baked_states = baked_states;
    }
    pub fn with_baked_states(mut self, baked_states: BakedStates) -> Self {
        self.set_baked_states(baked_states);
        self
    }

    pub fn set_layout(&mut self, layout: &'a B::PipelineLayout) {
        self.layout.replace(layout);
    }
    pub fn with_layout(mut self, layout: &'a B::PipelineLayout) -> Self {
        self.set_layout(layout);
        self
    }

    pub fn set_subpass(&mut self, subpass: Subpass<'a, B>) {
        self.subpass.replace(subpass);
    }
    pub fn with_subpass(mut self, subpass: Subpass<'a, B>) -> Self {
        self.set_subpass(subpass);
        self
    }

    pub fn set_flags(&mut self, flags: PipelineCreationFlags) {
        self.flags = flags;
    }
    pub fn with_flags(mut self, flags: PipelineCreationFlags) -> Self {
        self.set_flags(flags);
        self
    }

    pub fn set_parent(&mut self, parent: BasePipeline<'a, B::GraphicsPipeline>) {
        self.parent = match parent {
            BasePipeline::Pipeline(p) => BasePipelineExt::Pipeline(p),
            BasePipeline::Index(i) => BasePipelineExt::Index(i),
            BasePipeline::None => BasePipelineExt::None,
        };
    }
    pub fn with_parent(mut self, parent: BasePipeline<'a, B::GraphicsPipeline>) -> Self {
        self.set_parent(parent);
        self
    }

    // Helper Functions

    pub fn set_vertex_desc(&mut self, fmts: &[(VertexFormat, VertexInputRate)]) {
        let mut buffers = Vec::with_capacity(fmts.len());
        let mut attrs = Vec::with_capacity(fmts.len());

        for (i, (fmt, rate)) in fmts.into_iter().enumerate() {
            buffers.push(VertexBufferDesc {
                binding: i as u32,
                stride: fmt.stride,
                rate: *rate,
            });

            for (j, attr) in (&fmt.attributes).into_iter().enumerate() {
                attrs.push(AttributeDesc {
                    binding: i as u32,
                    location: j as u32,
                    element: *attr.element(),
                });
            }
        }

        self.set_vertex_buffers(buffers);
        self.set_attributes(attrs);
    }
    pub fn with_vertex_desc(mut self, fmts: &[(VertexFormat, VertexInputRate)]) -> Self {
        self.set_vertex_desc(fmts);
        self
    }

    pub fn set_framebuffer_size(&mut self, w: u32, h: u32) {
        let old = self.baked_states.clone();

        let rect = Rect {
            x: 0,
            y: 0,
            w: w as i16,
            h: h as i16,
        };

        self.set_baked_states(BakedStates {
            viewport: Some(Viewport {
                rect,
                depth: old.viewport.map_or(0.0..1.0, |v| v.depth),
            }),
            scissor: Some(rect),
            ..old
        })
    }
    pub fn with_framebuffer_size(mut self, w: u32, h: u32) -> Self {
        self.set_framebuffer_size(w, h);
        self
    }

    pub fn set_depth_test(&mut self, test: DepthTest) {
        self.depth_stencil.depth = Some(test)
    }
    pub fn with_depth_test(mut self, test: DepthTest) -> Self {
        self.set_depth_test(test);
        self
    }

    pub fn set_blend_targets(&mut self, targets: Vec<ColorBlendDesc>) {
        self.blender.targets = targets;
    }
    pub fn with_blend_targets(mut self, targets: Vec<ColorBlendDesc>) -> Self {
        self.set_blend_targets(targets);
        self
    }
}

//#[derive(Clone)]
pub struct PipelinesBuilder<'a, B: Backend> {
    builders: Vec<PipelineDescBuilder<'a, B>>,
}

impl<'a, B: Backend> Default for PipelinesBuilder<'a, B> {
    fn default() -> Self {
        Self {
            builders: Vec::new(),
        }
    }
}

impl<'a, B: Backend> PipelinesBuilder<'a, B> {
    pub fn add_pipeline(&mut self, builder: PipelineDescBuilder<'a, B>) {
        self.builders.push(builder);
    }
    pub fn with_pipeline(mut self, builder: PipelineDescBuilder<'a, B>) -> Self {
        self.add_pipeline(builder);
        self
    }

    pub fn add_child_pipeline(&mut self, index: usize, builder: PipelineDescBuilder<'a, B>) {
        self.builders[index].flags |= PipelineCreationFlags::ALLOW_DERIVATIVES;
        self.builders
            .push(builder.with_parent(BasePipeline::Index(index)));
    }
    pub fn with_child_pipeline(
        mut self,
        index: usize,
        builder: PipelineDescBuilder<'a, B>,
    ) -> Self {
        self.add_child_pipeline(index, builder);
        self
    }

    pub fn build(self, factory: &Factory<B>) -> Vec<B::GraphicsPipeline> {
        let mut pipelines = unsafe {
            factory.device().create_graphics_pipelines(
                self.builders.into_iter().map(|b| b.build(factory)),
                None,
            )
        };

        pipelines.into_iter().map(|p| p.unwrap()).collect()
    }
}
