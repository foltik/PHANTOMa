#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;

mod component;
mod error;
mod init;

#[allow(unused_imports)]
use rendy::{
    command::Families,
    factory::{self, Factory},
    graph::{present::PresentNode, render::*, Graph, GraphBuilder, ImageId},
    hal::{
        self,
        command::{ClearColor, ClearValue},
        window::PresentMode,
        Backend,
    },
    init::winit::{
        dpi::PhysicalSize,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    },
    init::AnyWindowedRendy,
    wsi::Surface,
};

use std::sync::{Arc, Mutex};

use component::{cube, filter, ComponentState};

fn create_image<B: Backend>(
    factory: &Factory<B>,
    builder: &mut GraphBuilder<B, Arc<Mutex<ComponentState>>>,
    surface: &Surface<B>,
    size: &PhysicalSize<u32>,
    clear: Option<ClearValue>,
) -> ImageId {
    builder.create_image(
        hal::image::Kind::D2(size.width, size.height, 1, 1),
        1,
        factory.get_surface_format(surface),
        clear,
    )
}

fn build_graph<B: Backend>(
    args: &init::Args,
    factory: &mut Factory<B>,
    families: &mut Families<B>,
    window: &Window,
    state: &Arc<Mutex<ComponentState>>,
) -> Graph<B, Arc<Mutex<ComponentState>>> {
    let mut graph_builder = GraphBuilder::new();

    let surface = factory.create_surface(window).unwrap();
    let size = window.inner_size();

    log::debug!("Creating {}x{} surface", size.width, size.height);

    let white = ClearValue {
        color: ClearColor {
            float32: [1.0, 1.0, 1.0, 1.0],
        },
    };

    let color = create_image(factory, &mut graph_builder, &surface, &size, None);
    let mesh = create_image(factory, &mut graph_builder, &surface, &size, Some(white));

    let cube = graph_builder.add_node(
        cube::TriangleDesc::default()
            .builder()
            .into_subpass()
            .with_color(mesh)
            .into_pass(),
    );

    let post = graph_builder.add_node(
        filter::FilterDesc::default()
            .builder()
            .with_image(mesh)
            .with_dependency(cube)
            .into_subpass()
            .with_color(color)
            .into_pass(),
    );

    let present = PresentNode::builder(&factory, surface, color)
        .with_dependency(post)
        .with_present_modes_priority(|m| match args.vsync {
            true => match m {
                PresentMode::RELAXED => Some(2),
                PresentMode::FIFO => Some(1),
                _ => Some(0),
            },
            false => match m {
                PresentMode::MAILBOX => Some(2),
                PresentMode::IMMEDIATE => Some(1),
                _ => Some(0),
            },
        });

    log::debug!(
        "Creating {} image swapchain with present mode {:?}",
        present.image_count(),
        present.present_mode()
    );

    graph_builder.add_node(present);

    graph_builder.build(factory, families, &state).unwrap()
}

fn run<B: Backend>(
    args: init::Args,
    event_loop: EventLoop<()>,
    mut factory: Factory<B>,
    mut families: Families<B>,
    window: Window,
) {
    let started = std::time::Instant::now();
    let ms = |i: &std::time::Instant| {
        let elapsed = i.elapsed();
        elapsed.as_secs() as f64 * 1_000.0 + elapsed.subsec_nanos() as f64 / 1_000_000.0
    };

    let size = window.inner_size();

    let state = Arc::new(Mutex::new(ComponentState {
        frame: 0,
        t: 0.0,
        w: size.width,
        h: size.height,
        aspect: size.width as f32 / size.height as f32,
    }));

    let mut graph = Some(build_graph(
        &args,
        &mut factory,
        &mut families,
        &window,
        &state,
    ));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(new) => {
                    graph.take().unwrap().dispose(&mut factory, &state);

                    {
                        let mut state = state.lock().unwrap();
                        state.w = new.width;
                        state.h = new.height;
                        state.aspect = state.w as f32 / state.h as f32;
                    }

                    graph = Some(build_graph(
                        &args,
                        &mut factory,
                        &mut families,
                        &window,
                        &state,
                    ));
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                let ms = ms(&started);

                {
                    let mut state = state.lock().unwrap();
                    state.frame += 1;
                    state.t = ms;
                }

                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                factory.maintain(&mut families);

                let before = std::time::Instant::now();

                if let Some(ref mut graph) = graph {
                    graph.run(&mut factory, &mut families, &state);
                }

                let frame = { state.lock().unwrap().frame };
                let ms = ms(&before);

                log::trace!("Frame {}: {} ms", frame, ms);
            }
            _ => {}
        }

        if *control_flow == ControlFlow::Exit {
            let elapsed = started.elapsed();
            let elapsed_ns = elapsed.as_secs() * 1_000_000_000 + elapsed.subsec_nanos() as u64;

            graph.take().unwrap().dispose(&mut factory, &state);

            let frame = { state.lock().unwrap().frame };

            log::info!(
                "Elapsed: {:?}. Frames: {}. FPS: {}",
                elapsed,
                frame,
                frame as u64 * 1_000_000_000 / elapsed_ns
            );
        }
    });
}

fn main() {
    let config: factory::Config = Default::default();
    let event_loop = EventLoop::new();

    let args = init::args(&event_loop);

    env_logger::Builder::from_default_env()
        .filter_module("phantoma", args.log_level)
        .init();

    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize {
            width: 960,
            height: 640,
        })
        .with_fullscreen(args.fullscreen.clone())
        .with_title("PHANTOMa");

    let rendy = AnyWindowedRendy::init_auto(&config, window, &event_loop).unwrap();
    rendy::with_any_windowed_rendy!((rendy)
        (factory, families, _surface, window) => {
            run(args, event_loop, factory, families, window);
        }
    );
}
