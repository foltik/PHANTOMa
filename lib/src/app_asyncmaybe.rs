use std::sync::Arc;
use std::time::Instant;

use async_std::task;
use crossbeam_queue::SegQueue;
use async_std::sync::Mutex;

use crate::gfx::wgpu;
use crate::window::WindowBuilder;

struct App {
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
        1000.0 / self.dt
    }
}

// TODO: Keyboard/Mouse/(Midi)? event handling
use crate::async_closure::{AsyncFn1, AsyncFnMut3};
pub fn run<M, ModelFn, EventFn, UpdateFn, ViewFn>(
    model_fn: ModelFn,
    mut event_fn: EventFn,
    mut update_fn: UpdateFn,
    mut view_fn: ViewFn,
) where
    M: 'static + Send,
    ModelFn: for<'a> AsyncFn1<&'a App, Output = M> + Send,
    EventFn: for<'a, 'e> AsyncFnMut3<&'a App, &'a mut M, &'e winit::event::WindowEvent<'static>> + 'static + Send,
    UpdateFn: for<'a> AsyncFnMut3<&'a App, &'a mut M, f32> + 'static + Send,
    ViewFn: for<'a> AsyncFnMut3<&'a mut App, &'a M, &'a wgpu::SwapChainTextureView> + 'static + Send,
{
    init_logging();

    let instance = wgpu::Instance::new(wgpu::defaults::backends());

    let event_loop = winit::event_loop::EventLoop::new();

    async_scoped::scope_and_block(async move |scope| {
        let (mut window, adapter, device, queue) = WindowBuilder::default()
            // .fullscreen(&event_loop)
            .build(&event_loop, &instance)
            .await;
        let device = Arc::new(device);
        log::info!("Using device '{}'", adapter.get_info().name);

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

        // Begin polling the device
        async_std::task::spawn(async move {
            loop {
                device.poll(wgpu::Maintain::Wait);
            }
        });

        let model = model_fn.call_once(&app).await;

        // Track the moment the loop starts and the last update.
        let start = Instant::now();
        let mut last = start;

        // Wrap the `model` in an `Option`, allowing us to take full ownership within the `event_loop`
        // on `exit`.
        let mut model = Some(model);

        let event_q = Arc::new(SegQueue::<EventAsync>::new());
        let event_prod = Arc::clone(&event_q);

        let redraw_q = Arc::new(SegQueue::<()>::new());
        let redraw_prod = Arc::clone(&redraw_q);

        let update_loop = scope.spawn(async move {
            while !redraw_q.is_empty() {
                if let Some(model) = model.as_mut() {
                    // Update the app's times.
                    let now = Instant::now();

                    let since_start = now.duration_since(start);
                    let since_last = now.duration_since(last);
                    app.t = since_start.as_secs_f32();
                    app.dt = since_last.as_secs_f32();

                    let dt = app.dt;

                    // User update function.
                    update_fn.call_mut(&app, model, dt).await;
                    log::trace!("Updated in {:?}", now.elapsed());

                    last = now;
                }
                if let Some(model) = model.as_mut() {
                    if let Some(view) = window.swap_chain.next_frame() {
                        view_fn.call_mut(&mut app, model, &view).await;
                        app.staging.recall().await;
                    }
                }

                // let mut redraws = events.redraw_requests().await;
                // while redraws.next().await.is_some() {
                //     if let Some(model) = model.as_ref() {
                //         if let Some(view) = window.swap_chain.next_frame() {
                //             view_fn.call_mut(&mut app, &model, &view).await;
                //             app.staging.recall().await;
                //         }
                //     }
                // }
            }
        });

        // Run the event loop.
        use crate::window::async_ext::EventAsync;
        use crate::window::async_ext::EventLoopAsync as _;
        use winit::event::WindowEvent;
        event_loop.run_async(async move |mut runner| loop {

            runner.wait().await;

            let mut events = runner.recv_events().await;
            while let Some(event) = events.next().await {
                // if let EventAsync::WindowEvent(event) = event {
                //     let window = &mut window;
                //     if let WindowEvent::Resized(size) = event {
                //         if size != window.size {
                //             // log::debug!("WindowEvent: Resized({:?})", size);

                //             app.width = size.width;
                //             app.height = size.height;
                //             window.size = size;

                //             window.rebuild_swap_chain(&app.device, size);
                //         }
                //     }
                //     if let WindowEvent::CloseRequested = event {
                //         // log::debug!("WindowEvent: CloseRequested");
                //         return;
                //     }

                //     if let Some(model) = model.as_mut() {
                //         event_fn.call_mut(&app, model, &event).await;
                //     }
                // }
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
}
