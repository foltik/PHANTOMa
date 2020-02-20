mod pipeline;
mod shape;
mod uniform;

use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{Layout, PrepareResult, RenderGroup},
        BufferAccess, GraphContext, ImageAccess, NodeBuffer, NodeImage,
    },
    hal::{pass::Subpass, pso, Backend},
    shader::{ShaderSetBuilder, SpirvReflection},
};

use pipeline::PipelineDescBuilder;

pub struct ComponentState {
    pub frame: u32,
    pub t: f64,
    pub w: u32,
    pub h: u32,
    pub aspect: f32,
}

pub trait ComponentBuilder<B: Backend> {
    type For: RenderGroup<B, ComponentState>;

    fn depth(&self) -> bool {
        false
    }

    fn input_rate(&self) -> Option<pso::VertexInputRate> {
        Some(pso::VertexInputRate::Vertex)
    }

    fn buffers(&self) -> Vec<BufferAccess> {
        vec![]
    }

    fn images(&self) -> Vec<ImageAccess> {
        vec![]
    }

    fn shaders(&self) -> &'static ShaderSetBuilder;

    fn layout(&self, reflect: &SpirvReflection) -> Layout {
        reflect.layout().unwrap()
    }

    fn pipeline_builder<'a>(
        &self,
        factory: &Factory<B>,
        builder: PipelineDescBuilder<'a, B>,
    ) -> PipelineDescBuilder<'a, B>;

    fn build(
        self,
        ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        aux: &ComponentState,
        pipeline: B::GraphicsPipeline,
        layout: B::PipelineLayout,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
    ) -> Self::For;
}

pub trait Component<B: Backend> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        queue: QueueId,
        index: usize,
        subpass: Subpass<'_, B>,
        aux: &ComponentState,
    ) -> PrepareResult;

    fn draw(
        &mut self,
        encoder: RenderPassEncoder<'_, B>,
        index: usize,
        subpass: Subpass<'_, B>,
        aux: &ComponentState,
    );

    fn dispose(self: Box<Self>, factory: &mut Factory<B>);
}

macro_rules! component {
    ($builder:ident, $comp:ident) => {
        use rendy::graph::{BufferAccess, ImageAccess};

        impl<B: Backend> RenderGroupDesc<B, ComponentState> for $builder
        where
            $builder: ComponentBuilder<B>,
        {
            fn depth(&self) -> bool {
                ComponentBuilder::depth(self)
            }

            fn buffers(&self) -> Vec<BufferAccess> {
                ComponentBuilder::buffers(self)
            }

            fn images(&self) -> Vec<ImageAccess> {
                ComponentBuilder::images(self)
            }

            fn build(
                self,
                ctx: &GraphContext<B>,
                factory: &mut Factory<B>,
                queue: QueueId,
                aux: &ComponentState,
                framebuffer_width: u32,
                framebuffer_height: u32,
                subpass: Subpass<'_, B>,
                buffers: Vec<NodeBuffer>,
                images: Vec<NodeImage>,
            ) -> Result<Box<dyn RenderGroup<B, ComponentState>>, pso::CreationError> {
                let shader_set = ComponentBuilder::shaders(&self);

                let reflect = shader_set.reflect().unwrap();
                let mut shaders = shader_set.build(factory, Default::default()).unwrap();

                let layout = ComponentBuilder::layout(&self, &reflect);

                let set_layouts = layout
                    .sets
                    .into_iter()
                    .map(|set| factory.create_descriptor_set_layout(set.bindings).unwrap())
                    .collect::<Vec<_>>();

                let pipeline_layout = unsafe {
                    factory
                        .device()
                        .create_pipeline_layout(
                            set_layouts.iter().map(|l| l.raw()),
                            layout.push_constants,
                        )
                        .unwrap()
                };

                let mut pipeline = PipelineDescBuilder::default()
                    .with_shaders(shaders.raw().unwrap())
                    .with_layout(&pipeline_layout)
                    .with_subpass(subpass)
                    .with_framebuffer_size(framebuffer_width, framebuffer_height);

                match ComponentBuilder::input_rate(&self) {
                    Some(rate) => {
                        pipeline.set_vertex_desc(&[(reflect.attributes_range(..).unwrap(), rate)])
                    }
                    _ => {}
                };

                pipeline = ComponentBuilder::pipeline_builder(&self, factory, pipeline);

                let mut pipes = PipelinesBuilder::default()
                    .with_pipeline(pipeline)
                    .build(factory);

                shaders.dispose(factory);

                Ok(Box::new(ComponentBuilder::build(
                    self,
                    ctx,
                    factory,
                    queue,
                    aux,
                    pipes.remove(0),
                    pipeline_layout,
                    buffers,
                    images,
                )))
            }
        }

        impl<B: Backend> RenderGroup<B, ComponentState> for $comp<B>
        where
            $comp<B>: Component<B>,
        {
            fn prepare(
                &mut self,
                factory: &Factory<B>,
                queue: QueueId,
                index: usize,
                subpass: Subpass<'_, B>,
                aux: &ComponentState,
            ) -> PrepareResult {
                Component::prepare(self, factory, queue, index, subpass, aux)
            }

            fn draw_inline(
                &mut self,
                encoder: RenderPassEncoder<'_, B>,
                index: usize,
                subpass: Subpass<'_, B>,
                aux: &ComponentState,
            ) {
                Component::draw(self, encoder, index, subpass, aux)
            }

            fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &ComponentState) {
                Component::dispose(self, factory)
            }
        }
    };
}

pub mod filter;
pub mod triangle;
