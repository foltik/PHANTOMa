use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc},
        GraphContext, NodeBuffer, NodeImage
    },
    hal::{self, device::Device, pass::Subpass, pso, Backend},
    mesh::{AsVertex, Mesh, PosTex},
    shader::{ShaderKind, SourceLanguage, SourceShaderInfo, SpirvShader},
};

use nalgebra::{Matrix4, RealField, Vector3};

use glsl_layout::{mat4x4, AsStd140};

use std::convert::TryInto;

use super::{
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    shape::Shape,
    uniform::{DynamicUniform, PushConstant},
    Component, ComponentBuilder, ComponentState,
};
use rendy_shader::ShaderSetBuilder;

lazy_static! {
    static ref VERTEX: SpirvShader = SourceShaderInfo::new(
        include_str!("shader.vert"),
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/component/filter/shader.vert"
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
            "/src/component/filter/shader.frag"
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
pub struct FilterDesc {}

impl<B: Backend> ComponentBuilder<B> for FilterDesc {
    type For = Filter<B>;

    fn vertex_input(&self) -> bool {
        false
    }

    fn images(&self) -> Vec<ImageAccess> {
        vec![ImageAccess {
            access: hal::image::Access::SHADER_READ,
            usage: hal::image::Usage::SAMPLED,
            layout: hal::image::Layout::ShaderReadOnlyOptimal,
            stages: pso::PipelineStage::FRAGMENT_SHADER
        }]
    }

    fn shaders(&self) -> &'static ShaderSetBuilder {
        &SHADERS
    }

    fn build_pipeline<'a>(
        &self,
        factory: &Factory<B>,
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
        queue: QueueId,
        aux: &ComponentState,
        pipeline: B::GraphicsPipeline,
        layout: B::PipelineLayout,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
    ) -> Self::For {
        println!("{:#?}", images);
        let image = images.get(0);

        let layout_binding = factory
            .create_descriptor_set_layout(vec![pso::DescriptorSetLayoutBinding {
                binding: 0,
                ty: pso::DescriptorType::SampledImage,
                count: 1,
                stage_flags: pso::ShaderStageFlags::VERTEX,
                immutable_samplers: false,
            }])
            .unwrap();

        let buffer = factory
            .create_buffer(
                BufferInfo {
                    size: std::mem::size_of::<T::Std140>() as u64,
                    usage: hal::buffer::Usage::UNIFORM,
                },
                rendy::memory::Dynamic,
            )
            .unwrap();

        let set = factory.create_descriptor_set(layout_binding.clone()).unwrap();
        let desc = pso::Descriptor::Buffer(buffer.raw(), None..None);
        unsafe {
            let set = set.raw();
            factory.write_descriptor_sets(Some(pso::DescriptorSetWrite {
                set,
                binding: 0,
                array_offset: 0,
                descriptors: Some(desc),
            }));
        }

        Filter::<B> {
            pipeline,
            layout,
            cfg: FilterCfg::new(aux),
        }
    }
}

fn convert_matrix(mat: &Matrix4<f32>) -> mat4x4 {
    let flat: [f32; 16] = mat.as_slice().try_into().unwrap();
    let arr: [[f32; 4]; 4] = unsafe { std::mem::transmute(flat) };
    arr.into()
}

#[derive(Clone, Copy, AsStd140, Debug)]
pub struct FilterPush {
    transform: mat4x4,
}

impl std::default::Default for FilterPush {
    fn default() -> Self {
        Self {
            transform: convert_matrix(&Matrix4::identity()),
        }
    }
}

#[derive(Debug)]
struct FilterCfg {
}

impl FilterCfg {
    fn new(aux: &ComponentState) -> Self {
        Self {
        }
    }
}

#[derive(Debug)]
pub struct Filter<B: Backend> {
    pipeline: B::GraphicsPipeline,
    layout: B::PipelineLayout,
    cfg: FilterCfg,
}

impl<B: Backend> Component<B> for Filter<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: Subpass<'_, B>,
        aux: &ComponentState,
    ) -> PrepareResult {

        PrepareResult::DrawReuse
    }

    fn draw(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: Subpass<'_, B>,
        _aux: &ComponentState,
    ) {
        encoder.bind_graphics_pipeline(&self.pipeline);

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
