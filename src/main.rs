#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate failure;

mod component;
mod error;

use rendy::{
    command::{Families, QueueId, RenderPassEncoder},
    factory::{Config, Factory},
    graph::{render::*, Graph, GraphBuilder, GraphContext, NodeBuffer, NodeImage},
    hal::{self, Backend},
    init::winit::{
        dpi::PhysicalSize,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    init::AnyWindowedRendy,
    memory::Dynamic,
    mesh::PosColor,
    resource::{Buffer, BufferInfo, DescriptorSetLayout, Escape, Handle},
    shader::{ShaderKind, SourceLanguage, SourceShaderInfo, SpirvReflection, SpirvShader},
};

use component::{
    ComponentState,
    triangle
};

fn run<B: Backend>(
    event_loop: EventLoop<()>,
    mut factory: Factory<B>,
    mut families: Families<B>,
    mut state: ComponentState,
    graph: Graph<B, ComponentState>
) {
    let started = std::time::Instant::now();

    let mut frame = 0u64;
    let mut elapsed = started.elapsed();
    let mut graph = Some(graph);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            Event::MainEventsCleared => {
                //elapsed = started.elapsed();
                //if elapsed >= std::time::Duration::new(5, 0) {
                //    *control_flow = ControlFlow::Exit
                //}

                frame += 1;
            }
            Event::RedrawRequested(_) => {
                factory.maintain(&mut families);
                if let Some(ref mut graph) = graph {
                    graph.run(&mut factory, &mut families, &state);
                }
            }
            _ => {}
        }

        if *control_flow == ControlFlow::Exit && graph.is_some() {
            let elapsed_ns = elapsed.as_secs() * 1_000_000_000 + elapsed.subsec_nanos() as u64;

            log::info!(
                "Elapsed: {:?}. Frames: {}. FPS: {}",
                elapsed,
                frame,
                frame * 1_000_000_000 / elapsed_ns
            );

            graph.take().unwrap().dispose(&mut factory, &state);
        }
    });
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_module("phantoma", log::LevelFilter::Trace)
        .init();

    let (width, height) = (960, 640);

    let config: Config = Default::default();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize { width, height })
        .with_title("Rendy example");

    let state = ComponentState {
        frame: 0,
        t: 0.0
    };

    let rendy = AnyWindowedRendy::init_auto(&config, window, &event_loop).unwrap();
    rendy::with_any_windowed_rendy!((rendy)
        (mut factory, mut families, surface, _window) => {
            let mut graph_builder = GraphBuilder::<_, ComponentState>::new();

            let sub =
            //TriangleRenderPipeline::builder()
            triangle::TriangleDesc::default().builder()
                    .into_subpass()
                    .with_color_surface()
                    .into_pass()
                    .with_surface(
                        surface,
                        hal::window::Extent2D {
                            width,
                            height,
                        },
                        Some(hal::command::ClearValue {
                            color: hal::command::ClearColor {
                                float32: [1.0, 1.0, 1.0, 1.0],
                            },
                        }),
                    );
            graph_builder.add_node(sub);

            let graph = graph_builder
                .build(&mut factory, &mut families, &state)
                .unwrap();

            run(event_loop, factory, families, state, graph);
        }
    );
}
