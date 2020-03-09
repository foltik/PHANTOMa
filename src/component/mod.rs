pub mod pipeline;
pub mod shader;
pub mod shape;
pub mod uniform;

use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{Layout, PrepareResult, RenderGroup},
        BufferAccess, GraphContext, ImageAccess, NodeBuffer, NodeImage,
    },
    resource::{Handle, DescriptorSetLayout},
    hal::{pass::Subpass, pso, Backend},
    shader::{ShaderSetBuilder, SpirvReflection},
};

use std::sync::{Arc, Mutex};

use pipeline::PipelineDescBuilder;

pub struct ComponentState {
    pub frame: u32,
    pub t: f64,

    pub w: u32,
    pub h: u32,
    pub aspect: f32,

    pub nyq: f32,
    pub amp: f32,
    pub fft: Vec<f32>,
}

pub trait ComponentBuilder<B: Backend> {
    type For: RenderGroup<B, Arc<Mutex<ComponentState>>>;

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
        aux: &Arc<Mutex<ComponentState>>,
        pipeline: B::GraphicsPipeline,
        layout: B::PipelineLayout,
        set_layouts: Vec<Handle<DescriptorSetLayout<B>>>,
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
        aux: &Arc<Mutex<ComponentState>>,
    ) -> PrepareResult;

    fn draw(
        &mut self,
        encoder: RenderPassEncoder<'_, B>,
        index: usize,
        subpass: Subpass<'_, B>,
        aux: &Arc<Mutex<ComponentState>>,
    );

    fn dispose(self: Box<Self>, factory: &mut Factory<B>);
}

macro_rules! component {
    ($builder:ident, $comp:ident) => {
        use rendy::graph::{BufferAccess, ImageAccess};
        use rendy::resource::{Handle, DescriptorSetLayout};
        use std::sync::{Arc, Mutex};

        impl<B: Backend> RenderGroupDesc<B, Arc<Mutex<ComponentState>>> for $builder
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
                aux: &Arc<Mutex<ComponentState>>,
                framebuffer_width: u32,
                framebuffer_height: u32,
                subpass: Subpass<'_, B>,
                buffers: Vec<NodeBuffer>,
                images: Vec<NodeImage>,
            ) -> Result<Box<dyn RenderGroup<B, Arc<Mutex<ComponentState>>>>, pso::CreationError> {
                let shader_set = ComponentBuilder::shaders(&self);

                let reflect = shader_set.reflect().unwrap();
                let mut shaders = shader_set.build(factory, Default::default()).unwrap();

                let layout = ComponentBuilder::layout(&self, &reflect);

                let set_layouts = layout
                    .sets
                    .into_iter()
                    .map(|set| factory.create_descriptor_set_layout(set.bindings).unwrap())
                    .map(Handle::from)
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
                    set_layouts,
                    buffers,
                    images,
                )))
            }
        }

        impl<B: Backend> RenderGroup<B, Arc<Mutex<ComponentState>>> for $comp<B>
        where
            $comp<B>: Component<B>,
        {
            fn prepare(
                &mut self,
                factory: &Factory<B>,
                queue: QueueId,
                index: usize,
                subpass: Subpass<'_, B>,
                aux: &Arc<Mutex<ComponentState>>,
            ) -> PrepareResult {
                Component::prepare(self, factory, queue, index, subpass, aux)
            }

            fn draw_inline(
                &mut self,
                encoder: RenderPassEncoder<'_, B>,
                index: usize,
                subpass: Subpass<'_, B>,
                aux: &Arc<Mutex<ComponentState>>,
            ) {
                Component::draw(self, encoder, index, subpass, aux)
            }

            fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &Arc<Mutex<ComponentState>>) {
                Component::dispose(self, factory)
            }
        }
    };
}
