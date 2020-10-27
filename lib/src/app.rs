use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Instant;

use crossbeam_queue::SegQueue;

use crate::gfx::wgpu;
use crate::window::WindowBuilder;

pub struct App {
    pub device: Arc<wgpu::Device>,
    pub(crate) queue: wgpu::Queue,
    pub(crate) staging: wgpu::util::StagingPool,

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

// TODO: Keyboard/Mouse/(Midi)? event handling
use crate::async_closure::{AsyncFn1, AsyncFnMut3, AsyncFnMut4};
pub fn run<M, ModelFn, InputFn, UpdateFn, ViewFn>(
    model_fn: ModelFn,
    mut input_fn: InputFn,
    mut update_fn: UpdateFn,
    mut view_fn: ViewFn,
) where
    M: 'static,
    ModelFn: for<'a> AsyncFn1<&'a App, Output = M> + 'static + Send,
    InputFn: for<'a, 'e> AsyncFnMut4<&'a App, &'a mut M, KeyState, Key> + 'static + Send,
    UpdateFn: for<'a> AsyncFnMut3<&'a App, &'a mut M, f32> + 'static + Send,
    ViewFn:
        for<'a> AsyncFnMut3<&'a mut App, &'a mut M, &'a wgpu::SwapChainTextureView> + 'static + Send,
{
    init_logging();

    let instance = wgpu::Instance::new(wgpu::defaults::backends());

    let exit = Arc::new(AtomicBool::new(false));
    let exit_tx = Arc::clone(&exit);

    let event_tx = Arc::new(SegQueue::<winit::event::Event<'static, ()>>::new());
    let event_rx = Arc::clone(&event_tx);

    let event_loop = winit::event_loop::EventLoop::new();

    let (mut window, adapter, device, queue) = async_std::task::block_on(
        WindowBuilder::default()
            // .fullscreen(&event_loop)
            .build(&event_loop, &instance),
    );

    let device = Arc::new(device);
    log::info!("Using device '{}'", adapter.get_info().name);

    let poll_device = Arc::clone(&device);
    thread::spawn(move || loop {
        poll_device.poll(wgpu::Maintain::Wait);
    });

    thread::spawn(|| {
        async_std::task::block_on(async move {
            let staging = wgpu::util::StagingPool::new(0x100);
            let mut app = App {
                device: Arc::clone(&device),
                queue,
                staging,

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

                                window.rebuild_swap_chain(&app.device, size);
                            }
                        }

                        if let WindowEvent::CloseRequested = event {
                            exit_tx.store(true, Ordering::SeqCst);
                            // TODO: Cleanly exit?
                        }

                        if let WindowEvent::KeyboardInput { input, .. } = event {
                            if let Some(key) = input.virtual_keycode {
                                input_fn.call_mut(&app, &mut model, input.state, key).await;
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
                // log::trace!("FPS: {:?}", app.fps());

                // User update function.
                update_fn.call_mut(&app, &mut model, dt).await;

                last = now;
                // log::trace!("Updated in {:?}", now.elapsed());

                view_fn
                    .call_mut(&mut app, &mut model, &window.swap_chain.frame())
                    .await;

                app.staging.recall().await;
            }
        });
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
