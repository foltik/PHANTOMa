use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc},
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{self, pso, device::Device, pass::Subpass, Backend},
    mesh::{AsVertex, Mesh, PosTex},
    shader::{ShaderKind, SourceLanguage, SourceShaderInfo, SpirvShader, SpirvReflection},
};

use super::{
    pipeline::{PipelinesBuilder, PipelineDescBuilder},
    shader
};

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

    static ref SHADERS: rendy::shader::ShaderSetBuilder = rendy::shader::ShaderSetBuilder::default()
        .with_vertex(&*VERTEX).unwrap()
        .with_fragment(&*FRAGMENT).unwrap();
}

#[derive(Clone, Debug)]
pub struct TriangleSettings {}

impl Default for TriangleSettings {
    fn default() -> Self {
        TriangleSettings {}
    }
}

#[derive(Clone, Debug)]
pub struct TriangleDesc {}

impl<B: Backend> RenderGroupDesc<B, ()> for TriangleDesc {
    fn build<'a>(
        self,
        ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        aux: &(),
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: Subpass<'_, B>,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, ()>>, pso::CreationError> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct Triangle<B: Backend> {
    pipeline: B::GraphicsPipeline,
    layout: B::PipelineLayout,
}

impl<B: Backend> RenderGroup<B, ()> for Triangle<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        queue: QueueId,
        index: usize,
        subpass: Subpass<'_, B>,
        aux: &(),
    ) -> PrepareResult {
        PrepareResult::DrawReuse
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        subpass: Subpass<'_, B>,
        aux: &(),
    ) {
        encoder.bind_graphics_pipeline(&self.pipeline);
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, aux: &()) {
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
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error> {
    let layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    let mut shaders = SHADERS.build(factory, Default::default()).unwrap();

    //let shader_vertex = unsafe { (*VERTEX)..module(factory).unwrap() };
    //let shader_fragment = unsafe { (*FRAGMENT).module(factory).unwrap() };

    let v = SHADER_REFLECT.attributes_range(..).unwrap().gfx_vertex_input_desc(hal::pso::VertexInputRate::Vertex);

    let pipes = PipelinesBuilder::default()
        .with_pipeline(
            PipelineDescBuilder::default()
                //.with_vertex_desc(&[(PosTex::vertex(), pso::VertexInputRate::Vertex)])
                /*.with_shaders(shader::shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))*/
                .with_layout(&layout)
                .with_subpass(subpass)
                //.with_framebuffer_size(framebuffer_width, framebuffer_height)
                /*.with_depth_test(pso::DepthTest {
                    fun: pso::Comparison::LessEqual,
                    write: false,
                })*/
                /*.with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: None,
                }]),*/
        )
        .build(factory);

    unsafe {
        shaders.dispose(factory);
        //factory.destroy_shader_module(shader_vertex);
        //factory.destroy_shader_module(shader_fragment);
    }

    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), layout)),
    }
}
