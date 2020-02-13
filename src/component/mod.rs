mod pipeline;
mod shape;
mod uniform;

use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup},
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{pass::Subpass, Backend},
};

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
        impl<B: Backend> RenderGroupDesc<B, ComponentState> for $builder
        where
            $builder: ComponentBuilder<B>,
        {
            fn depth(&self) -> bool {
                ComponentBuilder::depth(self)
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
                Ok(Box::new(ComponentBuilder::build(
                    self,
                    ctx,
                    factory,
                    queue,
                    aux,
                    framebuffer_width,
                    framebuffer_height,
                    subpass,
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

pub mod triangle;
