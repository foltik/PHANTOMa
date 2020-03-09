use glsl_layout::{AsStd140, Std140};
use rendy::{
    command::{QueueId, RenderPassEncoder},
    descriptor::DescriptorType,
    factory::Factory,
    graph::{
        render::{Layout, PrepareResult, RenderGroup, RenderGroupDesc, SetLayout},
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{
        device::Device,
        pass::Subpass,
        pso::{self, DescriptorSetLayoutBinding},
        Backend,
    },
    mesh::{AsVertex, Mesh, PosTex},
    shader::SpirvReflection,
};

use crate::component::{
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    shader::{self, Shader, ShaderKind, ShaderSetBuilder},
    shape::Shape,
    uniform::DynamicUniform,
    Component, ComponentBuilder, ComponentState,
};
use failure::_core::fmt::Formatter;

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

#[derive(Debug)]
struct Band {
    lo: f32,
    ctr: f32,
    hi: f32,
}

fn octave_bands(nyq: f32, n: f32, ctr: f32) -> Vec<Band> {
    let fd = 2.0_f32.powf(1.0 / (2.0 * n));
    let start: i32 = (ctr / 1000.0).log2() as i32 * n as i32;

    (start..)
        .map(|f| 1000.0 * (2.0_f32.powf(f as f32 / n)))
        .take_while(|f| f * fd < nyq)
        .map(|f| Band {
            lo: f / fd,
            ctr: f,
            hi: f * fd,
        })
        .collect()
}

fn energy(fft: &[f32], nyq: f32, lo: f32, hi: f32) -> f32 {
    let i = ((lo / nyq) * fft.len() as f32).round() as usize;
    let j = ((hi / nyq) * fft.len() as f32).round() as usize;

    &fft[i..=j].iter().sum::<f32>() / (j - i + 1) as f32
}

fn band_energy<'a>(fft: &'a [f32], nyq: f32, bands: &'a [Band]) -> impl Iterator<Item = f32> + 'a {
    bands.iter().map(move |b| energy(fft, nyq, b.lo, b.hi))
}

#[derive(Default, Debug)]
pub struct OctavesDesc {}

impl<B: Backend> ComponentBuilder<B> for OctavesDesc {
    type For = Octaves<B>;

    fn shaders(&self) -> &'static ShaderSetBuilder {
        &SHADERS
    }

    fn layout(&self, _reflect: &SpirvReflection) -> Layout {
        Layout {
            sets: vec![SetLayout {
                bindings: vec![DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                    stage_flags: pso::ShaderStageFlags::VERTEX,
                    immutable_samplers: false,
                }],
            }],
            //sets: vec![],
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
        set_layouts: Vec<Handle<DescriptorSetLayout<B>>>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Self::For {
        let (nyq, fft_len) = {
            let aux = aux.lock().unwrap();
            (aux.nyq, aux.fft.len())
        };

        let len = octave_bands(nyq, 3.0, 15.625).len();

        //let s = 1.8 / fft_len as f32;
        let s = 1.8 / (2.0 * len as f32);
        let mesh = Shape::Plane(None)
            .generate::<Vec<PosTex>>(Some((s, s, s)))
            .build(queue, factory)
            .unwrap();

        let uniform_layout = set_layouts[0].clone();

        Octaves::<B> {
            pipeline,
            layout,
            mesh,
            push: OctavesPush::default(),
            ubo: DynamicUniform::new_from_layout(uniform_layout),
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, align(16))]
struct PushValue(f32);

#[derive(Copy, Clone)]
pub struct OctavesPush {
    n: PushValue,
    fft: [PushValue; 256],
}

unsafe impl Std140 for OctavesPush {}
unsafe impl AsStd140 for OctavesPush {
    type Align = glsl_layout::align::Align16;
    type Std140 = OctavesPush;

    fn std140(&self) -> Self::Std140
    where
        Self::Std140: Sized,
    {
        *self
    }
}

impl std::fmt::Debug for OctavesPush {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OctavesPush {{ &[f32; {}]... }}", self.fft.len())
    }
}

impl std::default::Default for OctavesPush {
    fn default() -> Self {
        Self {
            n: PushValue(0.0),
            fft: [PushValue(0.0); 256],
        }
    }
}

#[derive(Debug)]
pub struct Octaves<B: Backend> {
    pipeline: B::GraphicsPipeline,
    layout: B::PipelineLayout,
    mesh: Mesh<B>,
    push: OctavesPush,
    ubo: DynamicUniform<B, OctavesPush>,
}

impl<B: Backend> Component<B> for Octaves<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: Subpass<'_, B>,
        aux: &Arc<Mutex<ComponentState>>,
    ) -> PrepareResult {
        let aux = aux.lock().unwrap();

        /*
        let avg = aux.fft.iter().take(256).sum::<f32>() / aux.fft.len() as f32;
        let max = aux
            .fft
            .iter()
            .take(256)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();
        */

        /*
        let vals = aux
            .fft
            .iter()
            .take(256)
            .map(|v| PushValue(*v))
            .collect::<Vec<_>>();
        */

        //self.push.fft.copy_from_slice(&vals);

        let bands = octave_bands(aux.nyq, 3.0, 15.625);

        let energy = band_energy(&aux.fft, aux.nyq, &bands)
            .map(|v| PushValue(v))
            .collect::<Vec<_>>();

        self.push.n = PushValue(bands.len() as f32);

        for (i, e) in energy.iter().enumerate() {
            self.push.fft[i] = *e;
        }
        //self.push.fft.copy_from_slice(&energy);

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

        let f = vec![1.0_f32];

        self.ubo.bind(index, &self.layout, 0, &mut encoder);

        let n = self.push.n.0 as u32;

        self.mesh
            .bind_and_draw(0, &[PosTex::vertex()], 0..n, &mut encoder)
            .unwrap();
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory.device().destroy_pipeline_layout(self.layout);
        }
    }
}

component!(OctavesDesc, Octaves);
