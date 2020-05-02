use glsl_layout::AsStd140;
use rendy::{
    command::{QueueId, RenderPassEncoder},
    descriptor::DescriptorType,
    factory::Factory,
    graph::{
        render::{Layout, PrepareResult, RenderGroup, RenderGroupDesc, SetLayout},
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{
        self,
        device::Device,
        pass::Subpass,
        pso::{self, DescriptorSetLayoutBinding},
        Backend,
    },
    memory::Write,
    resource::{
        self, Buffer, BufferInfo, DescriptorSet, Escape, ImageViewInfo, Sampler, SamplerDesc,
        ViewKind,
    },
    shader::SpirvReflection,
};

use crate::component::{
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    shader::{self, Shader, ShaderKind, ShaderSetBuilder},
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

#[derive(Default, Clone, Copy, AsStd140, Debug)]
pub struct GlitchPush {
    t: f32,
    aspect: f32,
    amp: f32,
}

#[derive(Default, Debug)]
pub struct GlitchDesc {}

impl<B: Backend> ComponentBuilder<B> for GlitchDesc {
    type For = Glitch<B>;

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

    fn layout(&self, _reflect: &SpirvReflection) -> Layout {
        println!("{:#?}", _reflect.layout().unwrap());

        Layout {
            sets: vec![SetLayout {
                bindings: vec![
                    DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: DescriptorType::UniformBuffer,
                        count: 1,
                        stage_flags: pso::ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                    DescriptorSetLayoutBinding {
                        binding: 1,
                        ty: DescriptorType::CombinedImageSampler,
                        count: 1,
                        stage_flags: pso::ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                ],
            }],
            push_constants: vec![],
        }
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
        set_layouts: Vec<Handle<DescriptorSetLayout<B>>>,
        _buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
    ) -> Self::For {
        let image = images.get(0).expect("Glitch requires an input image!");
        let set_layout = set_layouts.get(0).unwrap();

        let buffer_sz = std::mem::size_of::<<GlitchPush as AsStd140>::Std140>() as u64;
        let buffer = factory
            .create_buffer(
                BufferInfo {
                    size: buffer_sz,
                    usage: hal::buffer::Usage::UNIFORM,
                },
                rendy::memory::Dynamic,
            )
            .unwrap();

        let sampler = factory
            .create_sampler(SamplerDesc::new(
                resource::Filter::Nearest,
                resource::WrapMode::Clamp,
            ))
            .unwrap();

        let image_handle = ctx.get_image(image.id).unwrap();

        let view = factory
            .create_image_view(
                image_handle.clone(),
                ImageViewInfo {
                    view_kind: ViewKind::D2,
                    format: image_handle.info().format,
                    swizzle: hal::format::Swizzle::NO,
                    range: image.range.clone(),
                },
            )
            .unwrap();

        let set = factory.create_descriptor_set(set_layout.clone()).unwrap();

        unsafe {
            let set = set.raw();
            factory.write_descriptor_sets(vec![
                hal::pso::DescriptorSetWrite {
                    set,
                    binding: 0,
                    array_offset: 0,
                    descriptors: Some(hal::pso::Descriptor::Buffer(buffer.raw(), None..None)),
                },
                hal::pso::DescriptorSetWrite {
                    set,
                    binding: 1,
                    array_offset: 0,
                    descriptors: Some(hal::pso::Descriptor::CombinedImageSampler(
                        view.raw(),
                        hal::image::Layout::ShaderReadOnlyOptimal,
                        sampler.raw(),
                    )),
                },
            ]);
        }

        Glitch::<B> {
            pipeline,
            layout,
            set,
            buffer,
            sampler,
            push: GlitchPush::default(),
        }
    }
}

#[derive(Debug)]
pub struct Glitch<B: Backend> {
    pipeline: B::GraphicsPipeline,
    layout: B::PipelineLayout,
    set: Escape<DescriptorSet<B>>,
    buffer: Escape<Buffer<B>>,
    sampler: Escape<Sampler<B>>,
    push: GlitchPush,
}

impl<B: Backend> Component<B> for Glitch<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        _index: usize,
        _subpass: Subpass<'_, B>,
        aux: &Arc<Mutex<ComponentState>>,
    ) -> PrepareResult {
        {
            let aux = aux.lock().unwrap();
            self.push.t = aux.t as f32;
            self.push.aspect = aux.aspect;
            self.push.amp = aux.amp;
            println!("{}", aux.amp);
        }

        let sz = std::mem::size_of::<<GlitchPush as AsStd140>::Std140>();

        let bytes = unsafe {
            std::slice::from_raw_parts(
                &self.push.std140() as *const <GlitchPush as AsStd140>::Std140 as *const u8,
                sz,
            )
        };

        let range = 0..self.buffer.size();
        let mut mapped = self.buffer.map(factory.device(), range).unwrap();

        let mut writer = unsafe { mapped.write::<u8>(factory.device(), 0..sz as u64).unwrap() };

        let slice = unsafe { writer.slice() };

        slice.copy_from_slice(bytes);

        PrepareResult::DrawRecord
    }

    fn draw(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        _index: usize,
        _subpass: Subpass<'_, B>,
        _aux: &Arc<Mutex<ComponentState>>,
    ) {
        encoder.bind_graphics_pipeline(&self.pipeline);

        unsafe {
            encoder.bind_graphics_descriptor_sets(
                &self.layout,
                0,
                Some(self.set.raw()),
                std::iter::empty(),
            );
        }

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

component!(GlitchDesc, Glitch);
