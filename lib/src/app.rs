use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;
use std::cell::RefCell;

use tokio::task;
use crossbeam_queue::SegQueue;

use crate::gfx::wgpu;
use crate::gfx::frame::Frame;
use crate::window::WindowBuilder;

pub struct App {
    pub device: Arc<wgpu::Device>,
    pub(crate) queue: wgpu::Queue,
    // pub(crate) staging: wgpu::util::StagingPool,
    pub(crate) staging: RefCell<wgpu::util::StagingBelt>,

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

pub fn run<M, ModelFn, InputFn, UpdateFn, ViewFn>(
    model_fn: ModelFn,
    input_fn: InputFn,
    update_fn: UpdateFn,
    view_fn: ViewFn,
) where
    M: 'static,
    ModelFn: FnOnce(&App) -> M + Send + 'static,
    InputFn: Fn(&App, &mut M, KeyState, Key) + Send + 'static,
    UpdateFn: Fn(&App, &mut M, f32) + Send + 'static,
    // ViewFn: for<'a, 'frame> Fn(&'a App, &'a M, &'frame Frame, &'a wgpu::RawTextureView) + Send + 'static,
    ViewFn: Fn(&App, &mut M, &mut Frame, &wgpu::RawTextureView) + Send + 'static,
{
    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
        init_logging();

        let instance = wgpu::Instance::new(wgpu::defaults::backends());

        let exit = Arc::new(AtomicBool::new(false));
        let exit_tx = Arc::clone(&exit);

        let event_tx = Arc::new(SegQueue::<winit::event::Event<'static, ()>>::new());
        let event_rx = Arc::clone(&event_tx);

        let event_loop = winit::event_loop::EventLoop::new();

        let (mut window, adapter, device, queue) = WindowBuilder::default()
            .title("PHANTOMa")
            .build(&event_loop, &instance).await;

        let device = Arc::new(device);
        log::info!("Using device '{}'", adapter.get_info().name);

        let poll_device = Arc::clone(&device);

        task::spawn_blocking(move || loop {
            poll_device.poll(wgpu::Maintain::Wait);
        });

        task::spawn_blocking(move || {
            let staging = wgpu::util::StagingBelt::new(0x100);
            let mut app = App {
                device: Arc::clone(&device),
                queue,
                staging: RefCell::new(staging),

                width: window.size.width,
                height: window.size.height,

                frame: 0,
                t: 0.0,
                dt: 0.0,
            };

            let before_model = Instant::now();
            let mut model = model_fn(&app);
            log::debug!("Loaded model in {:?}", before_model.elapsed());

            // Track the moment the loop starts and the last update.
            let start = Instant::now();
            let mut last = start;

            loop {
                while let Some(event) = event_rx.pop() {
                    use winit::event::WindowEvent;
                    if let winit::event::Event::WindowEvent { event, .. } = event {
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
                            exit_tx.store(true, Ordering::SeqCst);
                            // TODO: Cleanly exit?
                        }

                        if let WindowEvent::KeyboardInput { input, .. } = event {
                            if let Some(key) = input.virtual_keycode {
                                input_fn(&app, &mut model, input.state, key);
                            }
                        }
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
                update_fn(&app, &mut model, dt);

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

                    let mut frame = Frame::new(&app);
                    view_fn(&app, &mut model, &mut frame, &view);
                    frame.submit();

                    surface.present();
                }

                futures::executor::block_on(app.staging.borrow_mut().recall());
            }
        });

        event_loop.run(move |event, _, flow| {
            event_tx.push(event.to_static().unwrap());

            use winit::event_loop::ControlFlow;
            if exit.load(Ordering::SeqCst) {
                *flow = ControlFlow::Exit;
            } else {
                *flow = ControlFlow::Poll;
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
