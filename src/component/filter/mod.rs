use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc},
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{self, device::Device, pass::Subpass, pso, Backend},
    resource::{self, SamplerDesc},
};

use super::{
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    shader::{self, Shader, ShaderKind, ShaderSetBuilder},
    uniform::Sampler,
    Component, ComponentBuilder, ComponentState,
};

lazy_static! {
    static ref VERTEX: Shader =
        shader::from_source(include_str!("shader.vert"), ShaderKind::Vertex);
    static ref FRAGMENT: Shader =
        shader::from_source(include_str!("shader.frag"), ShaderKind::Fragment);
    static ref SHADERS: ShaderSetBuilder = ShaderSetBuilder::default()
        .with_vertex(&*VERTEX)
        .unwrap()
        .with_fragment(&*FRAGMENT)
        .unwrap();
}

#[derive(Default, Debug)]
pub struct FilterDesc {}

impl<B: Backend> ComponentBuilder<B> for FilterDesc {
    type For = Filter<B>;

    fn input_rate(&self) -> Option<pso::VertexInputRate> {
        None
    }

    fn images(&self) -> Vec<ImageAccess> {
        vec![ImageAccess {
            access: hal::image::Access::SHADER_READ,
            usage: hal::image::Usage::SAMPLED,
            layout: hal::image::Layout::ShaderReadOnlyOptimal,
            stages: pso::PipelineStage::FRAGMENT_SHADER,
        }]
    }

    fn shaders(&self) -> &'static ShaderSetBuilder {
        &SHADERS
    }

    fn pipeline_builder<'a>(
        &self,
        _factory: &Factory<B>,
        builder: PipelineDescBuilder<'a, B>,
    ) -> PipelineDescBuilder<'a, B> {
        builder
            .with_rasterizer(pso::Rasterizer {
                polygon_mode: pso::PolygonMode::Fill,
                cull_face: pso::Face::FRONT,
                front_face: pso::FrontFace::CounterClockwise,
                depth_clamping: false,
                depth_bias: None,
                conservative: false,
            })
            .with_blend_targets(vec![pso::ColorBlendDesc {
                mask: pso::ColorMask::ALL,
                blend: None,
            }])
    }

    fn build(
        self,
        ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &Arc<Mutex<ComponentState>>,
        pipeline: B::GraphicsPipeline,
        layout: B::PipelineLayout,
        _buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
    ) -> Self::For {
        let image = images.get(0).expect("Filter requires an input image!");

        let sampler = Sampler::new(
            ctx,
            factory,
            image,
            SamplerDesc::new(resource::Filter::Nearest, resource::WrapMode::Clamp),
            pso::ShaderStageFlags::FRAGMENT,
        );

        Filter::<B> {
            pipeline,
            layout,
            sampler,
        }
    }
}

#[derive(Debug)]
pub struct Filter<B: Backend> {
    pipeline: B::GraphicsPipeline,
    layout: B::PipelineLayout,
    sampler: Sampler<B>,
}

impl<B: Backend> Component<B> for Filter<B> {
    fn prepare(
        &mut self,
        _factory: &Factory<B>,
        _queue: QueueId,
        _index: usize,
        _subpass: Subpass<'_, B>,
        _aux: &Arc<Mutex<ComponentState>>,
    ) -> PrepareResult {
        PrepareResult::DrawReuse
    }

    fn draw(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        _index: usize,
        _subpass: Subpass<'_, B>,
        _aux: &Arc<Mutex<ComponentState>>,
    ) {
        encoder.bind_graphics_pipeline(&self.pipeline);

        self.sampler.bind(&self.layout, 0, &mut encoder);

        unsafe {
            encoder.draw(0..3, 0..1);
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory.device().destroy_pipeline_layout(self.layout);
        }
    }
}

component!(FilterDesc, Filter);
