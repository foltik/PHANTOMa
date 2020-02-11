#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate failure;

mod component;
mod error;

#[allow(unused_imports)]
use rendy::{
    command::{Families},
    factory::{Config, Factory},
    graph::{render::*, Graph, GraphBuilder},
    hal::{self, Backend},
    init::winit::{
        dpi::PhysicalSize,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        monitor::{MonitorHandle, VideoMode},
        window::{Fullscreen, Window, WindowBuilder},
    },
    init::{AnyWindowedRendy},
};

use std::io::{stdin, stdout, Write};

use component::{triangle, ComponentState};

#[allow(dead_code)]
fn prompt_for_monitor(event_loop: &EventLoop<()>) -> MonitorHandle {
    for (num, monitor) in event_loop.available_monitors().enumerate() {
        println!("Monitor #{}: {:?}", num, monitor.name());
    }

    print!("Please write the number of the monitor to use: ");
    stdout().flush().unwrap();

    let mut num = String::new();
    stdin().read_line(&mut num).unwrap();
    let num = num.trim().parse().ok().expect("Please enter a number");
    let monitor = event_loop
        .available_monitors()
        .nth(num)
        .expect("Please enter a valid ID");

    println!("Using {:?}", monitor.name());

    monitor
}

#[allow(dead_code)]
fn prompt_for_video_mode(monitor: &MonitorHandle) -> VideoMode {
    for (i, video_mode) in monitor.video_modes().enumerate() {
        println!("Video mode #{}: {}", i, video_mode);
    }

    print!("Please write the number of the video mode to use: ");
    stdout().flush().unwrap();

    let mut num = String::new();
    stdin().read_line(&mut num).unwrap();
    let num = num.trim().parse().ok().expect("Please enter a number");
    let video_mode = monitor
        .video_modes()
        .nth(num)
        .expect("Please enter a valid ID");

    println!("Using {}", video_mode);

    video_mode
}

fn build_graph<B: Backend>(
    factory: &mut Factory<B>,
    families: &mut Families<B>,
    window: &Window,
    state: &ComponentState,
) -> Graph<B, ComponentState> {
    let mut graph_builder = GraphBuilder::new();

    let surface = factory.create_surface(window).unwrap();
    let size = window.inner_size();

    println!("Creating surface with size {}x{}", size.width, size.height);

    graph_builder.add_node(
        triangle::TriangleDesc::default()
            .builder()
            .into_subpass()
            .with_color_surface()
            .into_pass()
            .with_surface(
                surface,
                hal::window::Extent2D {
                    width: size.width,
                    height: size.height,
                },
                Some(hal::command::ClearValue {
                    color: hal::command::ClearColor {
                        float32: [1.0, 1.0, 1.0, 1.0],
                    },
                }),
            ),
    );

    graph_builder.build(factory, families, state).unwrap()
}

fn run<B: Backend>(
    event_loop: EventLoop<()>,
    mut factory: Factory<B>,
    mut families: Families<B>,
    window: Window,
) {
    let started = std::time::Instant::now();

    let size = window.inner_size();
    let mut state = ComponentState {
        frame: 0,
        t: 0.0,
        w: size.width,
        h: size.height,
        aspect: size.width as f32 / size.height as f32,
    };

    let mut graph = Some(build_graph(&mut factory, &mut families, &window, &state));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(new) => {
                    graph.take().unwrap().dispose(&mut factory, &state);

                    state.w = new.width;
                    state.h = new.height;
                    state.aspect = state.w as f32 / state.h as f32;

                    graph = Some(build_graph(&mut factory, &mut families, &window, &state));
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                state.frame += 1;

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

        if *control_flow == ControlFlow::Exit {
            let elapsed = started.elapsed();
            let elapsed_ns = elapsed.as_secs() * 1_000_000_000 + elapsed.subsec_nanos() as u64;

            graph.take().unwrap().dispose(&mut factory, &state);

            log::info!(
                "Elapsed: {:?}. Frames: {}. FPS: {}",
                elapsed,
                state.frame,
                state.frame as u64 * 1_000_000_000 / elapsed_ns
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

    /*
    let mon = prompt_for_monitor(&event_loop);
    let mode = prompt_for_video_mode(&mon);
    */

    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize {
            width: 960,
            height: 640,
        })
        //.with_fullscreen(Some(Fullscreen::Exclusive(mode)))
        .with_title("PHANTOMa");

    let rendy = AnyWindowedRendy::init_auto(&config, window, &event_loop).unwrap();
    rendy::with_any_windowed_rendy!((rendy)
        (factory, families, _surface, window) => {
            run(event_loop, factory, families, window);
        }
    );
}
