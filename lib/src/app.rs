use std::time::Instant;

use std::sync::Arc;
use tokio::task;
use tokio::sync::mpsc;

use crate::gfx::wgpu;
use crate::gfx::frame::Frame;
use crate::window::WindowBuilder;

use crate::window::async_ext::{EventLoopAsync, EventAsync as Event};
use winit::event::WindowEvent;

pub struct App {
    pub device: Arc<wgpu::Device>,
    pub(crate) queue: Arc<wgpu::Queue>,
    pub(crate) staging: Option<wgpu::util::StagingPool>,

    pub width: u32,
    pub height: u32,

    pub frame: u64,
    pub t: f32,
    pub dt: f32,
}

impl App {
    pub fn encoder(&self, label: &str) -> wgpu::CommandEncoder {
        let desc = wgpu::CommandEncoderDescriptor { label: Some(label) };
        self.device.create_command_encoder(&desc)
    }

    pub fn submit(&self, encoder: wgpu::CommandEncoder) {
        self.queue.submit(Some(encoder.finish()));
    }

    pub fn fps(&self) -> f32 {
        1.0 / self.dt
    }
}

pub use winit::event::ElementState as KeyState;
pub use winit::event::VirtualKeyCode as Key;

use crate::async_closure::*;

pub fn run<M, ModelFn, InputFn, UpdateFn, ViewFn>(
    model_fn: ModelFn,
    mut input_fn: InputFn,
    mut update_fn: UpdateFn,
    view_fn: ViewFn,
) where
    M: Send + 'static,
    ModelFn: for<'a> AsyncFn1<&'a App, Output = M> + 'static + Send,
    InputFn: for<'a, 'e> AsyncFnMut4<&'a App, &'a mut M, KeyState, Key> + 'static + Send,
    UpdateFn: for<'a> AsyncFnMut3<&'a App, &'a mut M, f32> + 'static + Send,
    ViewFn: Fn(&App, &mut M, &mut Frame, &wgpu::RawTextureView) + Send + 'static,
{
    init_logging();

    let instance = wgpu::Instance::new(wgpu::defaults::backends());

    let event_loop = winit::event_loop::EventLoop::new();
    let (event_tx, mut event_rx) = mpsc::channel(64);

    let (mut window, adapter, device, queue) = futures::executor::block_on(WindowBuilder::default()
        .title("PHANTOMa")
        .build(&event_loop, &instance));

    let _rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {

        task::spawn(async move {
            let device = Arc::new(device);
            log::info!("Using device '{}'", adapter.get_info().name);

            let poll_device = Arc::clone(&device);
            task::spawn_blocking(move || loop {
                poll_device.poll(wgpu::Maintain::Wait);
            });

            let mut app = App {
                device: Arc::clone(&device),
                queue: Arc::new(queue),
                staging: Some(wgpu::util::StagingPool::new(0x100)),

                width: window.size.width,
                height: window.size.height,

                frame: 0,
                t: 0.0,
                dt: 0.0,
            };

            let before_model = Instant::now();
            let mut model = model_fn.call_once(&app).await;
            log::debug!("Loaded model in {:?}", before_model.elapsed());

            // Track the moment the loop starts and the last update.
            let start = Instant::now();
            let mut last = start;

            loop {
                while let Ok(event) = event_rx.try_recv() {
                    match event {
                        Event::WindowEvent { event, .. } => {
                            let window = &mut window;

                            if let WindowEvent::Resized(size) = event {
                                if size != window.size {
                                    log::debug!("Window resized to {}x{}", size.width, size.height);

                                    app.width = size.width;
                                    app.height = size.height;
                                    window.size = size;

                                    window.surface.configure(&device, &wgpu::SurfaceConfiguration {
                                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                        format: wgpu::defaults::texture_format(),
                                        width: size.width,
                                        height: size.height,
                                        present_mode: wgpu::PresentMode::Mailbox,
                                    });
                                    // window.rebuild_swap_chain(&app.device, size);
                                }
                            }

                            if let WindowEvent::CloseRequested = event {
                                // FIXME: Cleanly exit?
                            }

                            if let WindowEvent::KeyboardInput { input, .. } = event {
                                if let Some(key) = input.virtual_keycode {
                                    // input_fn(&app, &mut model, input.state, key);
                                    input_fn.call_mut(&app, &mut model, input.state, key).await;
                                }
                            }
                        }
                        Event::DeviceEvent { .. } => {}
                        _ => {}
                    }
                }

                // Update the app's times.
                let now = Instant::now();

                let since_start = now.duration_since(start);
                let since_last = now.duration_since(last);
                app.t = since_start.as_secs_f32();
                app.dt = since_last.as_secs_f32();

                let dt = app.dt;
                // log::info!("FPS: {:?}", app.fps());

                // User update function.
                update_fn.call_mut(&app, &mut model, dt).await;

                last = now;
                log::trace!("Updated in {:?}", now.elapsed());

                if let Ok(surface) = window.surface.get_current_texture() {
                    let view = surface.texture.create_view(&wgpu::TextureViewDescriptor {
                        label: None,
                        format: Some(wgpu::defaults::texture_format()),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: 0,
                        array_layer_count: None,
                    });

                    let mut frame = Frame::new(Arc::clone(&app.device), Arc::clone(&app.queue), app.staging.take().unwrap());
                    view_fn(&mut app, &mut model, &mut frame, &view);
                    app.staging = Some(frame.submit());

                    surface.present();
                }

                // window.request_redraw();

                app.staging.as_mut().unwrap().recall().await;
            }
        });

        event_loop.run_async(async move |mut runner| {
            loop {
                runner.wait().await;

                let mut recv_events = runner.recv_events().await;
                while let Some(event) = recv_events.next().await {
                    if let Err(e) = event_tx.try_send(event) {
                        log::debug!("Dropped event: {:?}", e)
                    }
                }
            }
        });
    });
}

fn init_logging() {
    let args: Vec<String> = std::env::args().collect();

    // wtf?
    let module: String = std::env::current_exe()
        .unwrap()
        .file_name()
        .unwrap()
        .to_owned()
        .into_string()
        .unwrap();

    let mut app_level = "info";
    for s in args {
        match s.as_str() {
            "-v" => app_level = "debug",
            "-vv" => app_level = "trace",
            _ => {}
        }
    }

    if app_level != "info" {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let global_level = match std::env::var("RUST_LOG") {
        Ok(s) => format!(",{}", s),
        _ => "".to_string(),
    };

    std::env::set_var(
        "RUST_LOG",
        format!("lib={},{}={}{}", app_level, module, app_level, global_level),
    );

    pretty_env_logger::init();

    log::trace!("Set RUST_LOG={}", std::env::var("RUST_LOG").unwrap());
}
