use std::time::Instant;
use std::sync::Arc;

use crate::gfx::wgpu;
use crate::window::WindowBuilder;

pub struct App {
    pub device: Arc<wgpu::Device>,
    pub(crate) queue: wgpu::Queue,
    pub(crate) staging: wgpu::util::StagingPool,

    pub frame: u64,
    pub t: f32,
    pub dt: f32,
}

impl App {
    pub const ASSETS_DIRECTORY_NAME: &'static str = "assets";

    // Create a new `App`.
    pub(crate) fn new(
        device: Arc<wgpu::Device>,
        queue: wgpu::Queue,
        staging: wgpu::util::StagingPool,
    ) -> Self {
        Self {
            device,
            queue,
            staging,

            frame: 0,
            t: 0.0,
            dt: 0.0,
        }
    }

    pub fn fps(&self) -> f32 {
        1000.0 / self.dt
    }
}


// TODO: Keyboard/Mouse/(Midi)? event handling

use crate::async_closure::{AsyncFn1, AsyncFnMut3};
pub fn run<M, ModelFn, UpdateFn, ViewFn>(
    model_fn: ModelFn,
    mut update_fn: UpdateFn,
    mut view_fn: ViewFn,
) 
where
    M: 'static,
    ModelFn: for<'a> AsyncFn1<&'a App, Output = M>,
    UpdateFn: for<'a> AsyncFnMut3<&'a App, &'a mut M, f32> + 'static,
    ViewFn: for<'a> AsyncFnMut3<&'a mut App, &'a M, &'a wgpu::SwapChainTextureView> + 'static,
{
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

    let instance = wgpu::Instance::new(wgpu::defaults::backends());

    let event_loop = winit::event_loop::EventLoop::new();

    let inner = async move || {
        let (mut window, adapter, device, queue) = WindowBuilder::default().build(&event_loop, &instance).await;

        log::info!("Using device '{}'", adapter.get_info().name);

        let device = Arc::new(device);

        let staging = wgpu::util::StagingPool::new(0x100);

        // let app = App::new(proxy, window, instance, adapter, device, queue);
        let mut app = App::new(Arc::clone(&device), queue, staging);

        let model = model_fn.call_once(&app).await;

        // Track the moment the loop starts and the last update.
        let start = Instant::now();
        let mut last = start;

        // Wrap the `model` in an `Option`, allowing us to take full ownership within the `event_loop`
        // on `exit`.
        let mut model = Some(model);

        let poll = async move || {
            loop {
                device.poll(wgpu::Maintain::Wait);
            }
        };
        async_std::task::spawn(poll());

        // Run the event loop.
        use crate::window::async_ext::EventLoopAsync as _;
        use crate::window::async_ext::EventAsync;
        use winit::event::WindowEvent;
        event_loop.run_async(async move |mut runner| loop {
            runner.wait().await;

            let mut events = runner.recv_events().await;
            while let Some(event) = events.next().await {
                if let EventAsync::WindowEvent(event) = event {
                    let window = &mut window;
                    if let WindowEvent::Resized(size) = event {
                        window.size = size;
                        window.rebuild_swap_chain(&app.device, size);
                    }
                }
            }

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

                last = now;
            }

            window.request_redraw();

            let mut redraws = events.redraw_requests().await;
            while redraws.next().await.is_some() {
                if let Some(model) = model.as_ref() {
                    if let Some(view) = window.swap_chain.next_frame() {
                        view_fn.call_mut(&mut app, &model, &view).await;

                        app.staging.recall().await;
                    }
                }
            }
        });
    };

    async_std::task::block_on(inner());
}