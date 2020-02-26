use glsl_layout::{mat4x4, AsStd140};
use nalgebra::{Matrix4, RealField, Vector3};
use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc},
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{device::Device, pass::Subpass, pso, Backend},
    mesh::{AsVertex, Mesh, PosTex},
};
use std::convert::TryInto;

use crate::component::{
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    shader::{self, Shader, ShaderKind, ShaderSetBuilder},
    shape::Shape,
    uniform::{DynamicUniform, PushConstant},
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
pub struct TriangleDesc {}

impl<B: Backend> ComponentBuilder<B> for TriangleDesc {
    type For = Triangle<B>;

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
                cull_face: pso::Face::BACK,
                front_face: pso::FrontFace::Clockwise,
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
        queue: QueueId,
        aux: &Arc<Mutex<ComponentState>>,
        pipeline: B::GraphicsPipeline,
        layout: B::PipelineLayout,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Self::For {
        let mesh = Shape::Cube
            .generate::<Vec<PosTex>>(Some((0.5, 0.5, 0.5)))
            .build(queue, factory)
            .unwrap();

        let aspect = {
            aux.lock().unwrap().aspect
        };

        let proj = Matrix4::new_perspective(aspect, f32::frac_pi_2(), 0.001, 100.0);
        let view = Matrix4::new_translation(&Vector3::new(0.0, 0.0, -2.0));

        Triangle::<B> {
            pipeline,
            layout,
            mesh,
            view_proj: proj * view,
            push: PushConstant::new(TrianglePush::default(), 0, pso::ShaderStageFlags::VERTEX),
            ubo: DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX),
        }
    }
}

fn convert_matrix(mat: &Matrix4<f32>) -> mat4x4 {
    let flat: [f32; 16] = mat.as_slice().try_into().unwrap();
    let arr: [[f32; 4]; 4] = unsafe { std::mem::transmute(flat) };
    arr.into()
}

#[derive(Clone, Copy, AsStd140, Debug)]
pub struct TrianglePush {
    transform: mat4x4,
}

impl std::default::Default for TrianglePush {
    fn default() -> Self {
        Self {
            transform: convert_matrix(&Matrix4::identity()),
        }
    }
}

#[derive(Debug)]
pub struct Triangle<B: Backend> {
    pipeline: B::GraphicsPipeline,
    layout: B::PipelineLayout,
    mesh: Mesh<B>,
    view_proj: Matrix4<f32>,
    push: PushConstant<TrianglePush>,
    ubo: DynamicUniform<B, TrianglePush>,
}

impl<B: Backend> Component<B> for Triangle<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: Subpass<'_, B>,
        aux: &Arc<Mutex<ComponentState>>,
    ) -> PrepareResult {
        let t = {
            aux.lock().unwrap().t
        };

        let model = Matrix4::new_rotation(Vector3::new(
            t as f32 / 600.0,
            t as f32 / 400.0,
            t as f32 / 2000.0,
        ));
        self.push.transform = convert_matrix(&(self.view_proj.clone() * model));

        self.ubo.write(factory, index, &self.push);

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

        self.ubo.bind(index, &self.layout, 0, &mut encoder);

        self.mesh
            .bind_and_draw(0, &[PosTex::vertex()], 0..1, &mut encoder)
            .unwrap();
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory.device().destroy_pipeline_layout(self.layout);
        }
    }
}

component!(TriangleDesc, Triangle);
