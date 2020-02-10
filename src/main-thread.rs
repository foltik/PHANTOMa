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
        window::{Window, WindowBuilder},
    },
    init::AnyWindowedRendy,
    memory::Dynamic,
    mesh::PosColor,
    resource::{Buffer, BufferInfo, DescriptorSetLayout, Escape, Handle},
    shader::{ShaderKind, SourceLanguage, SourceShaderInfo, SpirvReflection, SpirvShader},
    wsi::Surface,
};

use component::{triangle, ComponentState};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 640;

fn run<B: Backend>(
    window: Window,
    event_loop: EventLoop<()>,
    mut factory: Factory<B>,
    mut families: Families<B>,
    mut state: ComponentState,
    graph: Graph<B, ComponentState>,
) {
    let started = std::time::Instant::now();

    let mut frame = 0u64;
    let mut graph = Some(graph);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            Event::MainEventsCleared => {
                state.frame += 1;
                frame += 1;

                window.request_redraw();
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
            let elapsed = started.elapsed();
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

fn build_graph<B: Backend>(
    factory: &Factory<B>,
    families: &Families<B>,
    surface: Surface<B>,
    window: Window,
) -> GraphBuilder<B, ComponentState> {
    let mut graph_builder = GraphBuilder::new();

    graph_builder.add_node(
        triangle::TriangleDesc::default()
            .builder()
            .into_subpass()
            .with_color_surface()
            .into_pass()
            .with_surface(
                surface,
                hal::window::Extent2D {
                    width: WIDTH,
                    height: HEIGHT,
                },
                Some(hal::command::ClearValue {
                    color: hal::command::ClearColor {
                        float32: [1.0, 1.0, 1.0, 1.0],
                    },
                }),
            ),
    );

    graph_builder
}

fn render<B: Backend>(
    active: Arc<AtomicBool>,
    state: Arc<Mutex<ComponentState>>,
    mut factory: Factory<B>,
    mut families: Families<B>,
    window: Window,
    surface: Surface<B>,
) {
    let mut graph_builder = build_graph(&factory, &families, surface, window);

    let mut graph = {
        let state = state.lock().unwrap();
        graph_builder
            .build(&mut factory, &mut families, &state)
            .unwrap()
    };

    while active.load(Ordering::SeqCst) {
        let state = state.lock().unwrap();

        factory.maintain(&mut families);
        graph.run(&mut factory, &mut families, &state);
    }

    {
        let state = state.lock().unwrap();
        graph.dispose(&mut factory, &state);
    }
}

fn update(active: Arc<AtomicBool>, state: Arc<Mutex<ComponentState>>, event_loop: EventLoop<()>) {
    let started = std::time::Instant::now();
    let mut frame = 0u64;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    active.store(false, Ordering::SeqCst);
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                let mut state = state.lock().unwrap();
                state.frame += 1;

                //window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                /*
                factory.maintain(&mut families);
                if let Some(ref mut graph) = graph {
                    graph.run(&mut factory, &mut families, &state);
                }
                */
            }
            _ => {}
        }

        if *control_flow == ControlFlow::Exit {
            let elapsed = started.elapsed();
            let elapsed_ns = elapsed.as_secs() * 1_000_000_000 + elapsed.subsec_nanos() as u64;

            log::info!(
                "Elapsed: {:?}. Frames: {}. FPS: {}",
                elapsed,
                frame,
                frame * 1_000_000_000 / elapsed_ns
            );
        }
    });
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_module("phantoma", log::LevelFilter::Trace)
        .init();

    let config: Config = Default::default();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize {
            width: WIDTH,
            height: HEIGHT,
        })
        .with_title("PHANTOMa");

    let active = Arc::new(AtomicBool::new(true));
    let state = Arc::new(Mutex::new(ComponentState { frame: 0, t: 0.0 }));

    let rendy = AnyWindowedRendy::init_auto(&config, window, &event_loop).unwrap();
    rendy::with_any_windowed_rendy!((rendy)
        (mut factory, mut families, surface, window) => {

            println!("{:?}", window.current_monitor())


            /*
            let render_handle = {
                let (active, state) = (Arc::clone(&active), Arc::clone(&state));
                thread::spawn(move || render(active, state, factory, families, window, surface))
            };
            */

            update(active, state, event_loop);

            //render_handle.join().unwrap();
            //run(window, event_loop, factory, families, state, graph);
        }
    );
}
