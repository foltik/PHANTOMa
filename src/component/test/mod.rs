use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc},
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{device::Device, pass::Subpass, pso, Backend},
};

use glsl_layout::AsStd140;

use super::{
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    shader::{self, Shader, ShaderKind, ShaderSetBuilder},
    Component, ComponentBuilder, ComponentState,
    uniform::{DynamicUniform},
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

#[derive(Default, Clone, Copy, AsStd140, Debug)]
pub struct TestPush {
    t: f32,
}

#[derive(Default, Debug)]
pub struct TestDesc {}

impl<B: Backend> ComponentBuilder<B> for TestDesc {
    type For = Test<B>;

    fn input_rate(&self) -> Option<pso::VertexInputRate> {
        None
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
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &Arc<Mutex<ComponentState>>,
        pipeline: B::GraphicsPipeline,
        layout: B::PipelineLayout,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Self::For {

        Test::<B> {
            pipeline,
            layout,
            uniform: DynamicUniform::new(factory, pso::ShaderStageFlags::FRAGMENT),
            push: TestPush { t: 0.0 }
        }
    }
}

#[derive(Debug)]
pub struct Test<B: Backend> {
    pipeline: B::GraphicsPipeline,
    layout: B::PipelineLayout,
    uniform: DynamicUniform<B, TestPush>,
    push: TestPush,
}

impl<B: Backend> Component<B> for Test<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: Subpass<'_, B>,
        aux: &Arc<Mutex<ComponentState>>,
    ) -> PrepareResult {
        let (t, w, h) = {
            let aux = aux.lock().unwrap();
            (aux.t, aux.w, aux.h)
        };

        self.push.t = t as f32;

        self.uniform.write(factory, index, &self.push);

        PrepareResult::DrawRecord
    }

    fn draw(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: Subpass<'_, B>,
        _aux: &Arc<Mutex<ComponentState>>,
    ) {
        encoder.bind_graphics_pipeline(&self.pipeline);

        self.uniform.bind(index, &self.layout, 0, &mut encoder);

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

component!(TestDesc, Test);
