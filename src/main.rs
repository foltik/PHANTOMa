//!
//! The mighty triangle example.
//! This examples shows colord triangle on white background.
//! Nothing fancy. Just prove that `rendy` works.
//!
use rendy::{
    command::Families,
    factory::{Config, Factory},
    graph::{Graph, GraphBuilder},
    hal::{self, Backend},
    init::winit::{
        dpi::PhysicalSize,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    init::AnyWindowedRendy,
};

mod component;
use component::{triangle::Triangle, Component};

fn run<B: Backend>(
    event_loop: EventLoop<()>,
    mut factory: Factory<B>,
    mut families: Families<B>,
    graph: Graph<B, ()>,
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
                elapsed = started.elapsed();
                if elapsed >= std::time::Duration::new(5, 0) {
                    *control_flow = ControlFlow::Exit
                }

                frame += 1;
            }
            Event::RedrawRequested(_) => {
                factory.maintain(&mut families);
                if let Some(ref mut graph) = graph {
                    graph.run(&mut factory, &mut families, &());
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

            graph.take().unwrap().dispose(&mut factory, &());
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
        .with_inner_size(PhysicalSize::new(width, height))
        .with_title("Rendy example");

    let rendy = AnyWindowedRendy::init_auto(&config, window, &event_loop).unwrap();
    rendy::with_any_windowed_rendy!((rendy)
        (mut factory, mut families, surface, _window) => {
            let mut graph_builder = GraphBuilder::<_, ()>::new();

            graph_builder.add_node(
                Triangle::builder()
                    .with_surface(
                        surface,
                        hal::window::Extent2D {
                            width: 200,
                            height: 200,
                        },
                        Some(hal::command::ClearValue {
                            color: hal::command::ClearColor {
                                float32: [1.0, 1.0, 1.0, 1.0],
                            },
                        }),
                    ),
            );

            let graph = graph_builder
                .build(&mut factory, &mut families, &())
                .unwrap();

            run(event_loop, factory, families, graph);
        }
    );
}
