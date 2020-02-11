use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc},
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{self, device::Device, pass::Subpass, pso, Backend},
    mesh::{AsVertex, Mesh, VertexFormat},
    shader::{ShaderKind, SourceLanguage, SourceShaderInfo, SpirvReflection, SpirvShader},
};

use super::{
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    push_constant::PushConstant,
    shape::Shape,
    Component, ComponentBuilder, ComponentState,
};

use rendy_core::types::vertex::PosTex;

use nalgebra::{Matrix4, Vector3};

lazy_static! {
    static ref SHADER_REFLECT: SpirvReflection = SHADERS.reflect().unwrap();
    static ref VERTEX: SpirvShader = SourceShaderInfo::new(
        include_str!("shader.vert"),
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/component/triangle/shader.vert"
        )
        .into(),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    )
    .precompile()
    .unwrap();
    static ref FRAGMENT: SpirvShader = SourceShaderInfo::new(
        include_str!("shader.frag"),
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/component/triangle/shader.frag"
        )
        .into(),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    )
    .precompile()
    .unwrap();
    static ref SHADERS: rendy::shader::ShaderSetBuilder =
        rendy::shader::ShaderSetBuilder::default()
            .with_vertex(&*VERTEX)
            .unwrap()
            .with_fragment(&*FRAGMENT)
            .unwrap();
}

#[derive(Default, Debug)]
pub struct TriangleDesc {}

impl<B: Backend> ComponentBuilder<B> for TriangleDesc {
    type For = Triangle<B>;

    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        aux: &ComponentState,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: Subpass<'_, B>,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
    ) -> Self::For {
        assert!(buffers.is_empty());
        assert!(images.is_empty());

        let (pipeline, layout) = build_triangle_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            vec![],
        );

        let mesh = Shape::Cube
            .generate::<Vec<PosTex>>(Some((0.5, 0.5, 0.5)))
            .build(queue, factory)
            .unwrap();

        Triangle::<B> {
            pipeline,
            layout,
            mesh,
            cfg: TriangleCfg::new(aux),
            push: PushConstants::default(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct PushConstants {
    transform: Matrix4<f32>,
}
unsafe impl PushConstant for PushConstants {}

impl std::default::Default for PushConstants {
    fn default() -> Self {
        Self {
            transform: Matrix4::identity(),
        }
    }
}

#[derive(Debug)]
struct TriangleCfg {
    projection: Matrix4<f32>,
}

impl TriangleCfg {
    fn new(aux: &ComponentState) -> Self {
        Self {
            projection: Matrix4::new_perspective(aux.aspect, 60.0, 0.001, 100.0),
        }
    }
}

#[derive(Debug)]
pub struct Triangle<B: Backend> {
    pipeline: B::GraphicsPipeline,
    layout: B::PipelineLayout,
    mesh: Mesh<B>,
    cfg: TriangleCfg,
    push: PushConstants,
}

impl<B: Backend> Component<B> for Triangle<B> {
    fn prepare(
        &mut self,
        _factory: &Factory<B>,
        _queue: QueueId,
        _index: usize,
        _subpass: Subpass<'_, B>,
        _aux: &ComponentState,
    ) -> PrepareResult {
        PrepareResult::DrawRecord
    }

    fn draw(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        _index: usize,
        _subpass: Subpass<'_, B>,
        aux: &ComponentState,
    ) {
        encoder.bind_graphics_pipeline(&self.pipeline);

        let model = Matrix4::new_rotation(Vector3::new(aux.frame as f32 / 60.0, aux.frame as f32 / 40.0, 0.0));
        let view = Matrix4::new_translation(&Vector3::new(0.0, 0.0, -1.0));

        self.push.transform = self.cfg.projection.clone() * view * model;

        unsafe {
            encoder.push_constants(
                &self.layout,
                pso::ShaderStageFlags::VERTEX,
                0,
                PushConstant::raw(&self.push),
            );
        }

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

fn build_triangle_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> (B::GraphicsPipeline, B::PipelineLayout) {
    let push_constants = SHADER_REFLECT.push_constants(None).unwrap();

    let layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, push_constants)
            .unwrap()
    };

    let mut shaders = SHADERS.build(factory, Default::default()).unwrap();

    let format: VertexFormat = SHADER_REFLECT.attributes_range(..).unwrap();
    let rate = pso::VertexInputRate::Vertex;

    let mut pipes = PipelinesBuilder::default()
        .with_pipeline(
            PipelineDescBuilder::default()
                .with_vertex_desc(&[(format, rate)])
                //.with_vertex_desc(&[(Position::vertex(), pso::VertexInputRate::Vertex)])
                .with_shaders(shaders.raw().unwrap())
                .with_rasterizer(pso::Rasterizer {
                    polygon_mode: pso::PolygonMode::Fill,
                    cull_face: pso::Face::BACK,
                    front_face: pso::FrontFace::Clockwise,
                    depth_clamping: false,
                    depth_bias: None,
                    conservative: false
                })
                .with_layout(&layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                /*
                .with_depth_test(pso::DepthTest {
                    fun: pso::Comparison::LessEqual,
                    write: false,
                })
                */
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: None,
                }]),
        )
        .build(factory);

    shaders.dispose(factory);

    /*
    TODO: Actually handle failure
    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), layout)),
    }
    */
    (pipes.remove(0), layout)
}

component!(TriangleDesc, Triangle);
