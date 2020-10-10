use std::time::Instant;

use crate::gfx::wgpu;
use crate::window::{Window, WindowBuilder, Proxy};

mod builder;
pub use builder::{Builder, ModelFn, WindowFn, UpdateFn, ViewFn, ExitFn};

pub struct App {
    proxy: Proxy,

    pub window: Window,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    //draw_state: DrawState,
    //pub(crate) ui: ui::Arrangement,
    //pub mouse: state::Mouse,
    //pub keys: state::Keys,

    pub frame: u64,
    pub t: f32,
    pub dt: f32,
}

impl App {
    pub const ASSETS_DIRECTORY_NAME: &'static str = "assets";

    // Create a new `App`.
    pub(crate) fn new(
        proxy: Proxy,
        window: Window,
        instance: wgpu::Instance,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Self {
        // let ui = ui::Arrangement::new();
        // let mouse = state::Mouse::new();
        // let keys = state::Keys::default();
        Self {
            proxy,
            window,
            instance,
            adapter,
            device,
            queue,

            // focused_window,
            // adapters,
            // windows,
            // draw_state,
            // ui,
            // mouse,
            // keys,
            // time,
            frame: 0,
            t: 0.0,
            dt: 0.0,
        }
    }

    // /// Find and return the absolute path to the project's `assets` directory.
    // ///
    // /// This method looks for the assets directory in the following order:
    // ///
    // /// 1. Checks the same directory as the executable.
    // /// 2. Recursively checks exe's parent directories (to a max depth of 5).
    // /// 3. Recursively checks exe's children directories (to a max depth of 3).
    // pub fn assets_path(&self) -> Result<PathBuf, find_folder::Error> {
    //     find_assets_path()
    // }

    // /// The path to the current project directory.
    // ///
    // /// The current project directory is considered to be the directory containing the cargo
    // /// manifest (aka the `Cargo.toml` file).
    // ///
    // /// **Note:** Be careful not to rely on this directory for apps or sketches that you wish to
    // /// distribute! This directory is mostly useful for local sketches, experiments and testing.
    // pub fn project_path(&self) -> Result<PathBuf, find_folder::Error> {
    //     find_project_path()
    // }

    /// A handle to the **App** that can be shared across threads.
    ///
    /// This can be used to "wake up" the **App**'s inner event loop.
    pub fn create_proxy(&self) -> Proxy {
        self.proxy.clone()
    }

    pub fn fps(&self) -> f32 {
        1000.0 / self.dt
    }
}


pub(crate) fn startup<M: 'static>(builder: Builder<M>) {
    let instance = wgpu::Instance::new(wgpu::defaults::backends());

    let event_loop = winit::event_loop::EventLoop::new();
    let proxy = Proxy::new(&event_loop);

    let mut window_builder = WindowBuilder::default();
    if let Some(window_fn) = builder.window {
        window_builder = (window_fn)(window_builder);
    }

    let (window, adapter, device, queue) = async_std::task::block_on(window_builder.build(&event_loop, &instance));

    let app = App::new(proxy, window, instance, adapter, device, queue);

    let model = (builder.model)(&app);

    run(
        event_loop,
        app,
        model,
        //builder.event,
        builder.update,
        builder.view,
        builder.exit,
    )
}

fn run<M: 'static>(
    event_loop: winit::event_loop::EventLoop<()>,
    mut app: App,
    model: M,
    //event_fn: Option<EventFn<M, E>>,
    update_fn: Option<UpdateFn<M>>,
    view_fn: Option<ViewFn<M>>,
    exit_fn: Option<ExitFn<M>>,
) {
    // Track the moment the loop starts and the last update.
    let start = Instant::now();
    let mut last = start;

    // Wrap the `model` in an `Option`, allowing us to take full ownership within the `event_loop`
    // on `exit`.
    let mut model = Some(model);

    // Create a staging belt to handle buffer uploads
    let mut staging = wgpu::util::StagingBelt::new(64);

    // Run the event loop.
    event_loop.run(move |mut event, _, control_flow| {
        let exit = false;

        match event {
            // Check to see if we need to emit an update and request a redraw.
            winit::event::Event::MainEventsCleared => {
                if let Some(model) = model.as_mut() {
                    // Update the app's times.
                    let now = Instant::now();

                    let since_start = now.duration_since(start);
                    let since_last = now.duration_since(last);
                    app.t = since_start.as_secs_f32();
                    app.dt = since_last.as_secs_f32();

                    let dt = app.dt;

                    // User update function.
                    if let Some(update_fn) = update_fn {
                        update_fn(&app, model, dt);
                    }

                    // Request redraw from window.
                    app.window.request_redraw();

                    last = now;
                }
            }

            // Request a frame from the user for the specified window.
            //
            // TODO: Only request a frame from the user if this redraw was requested following an
            // update. Otherwise, just use the existing intermediary frame.
            winit::event::Event::RedrawRequested(_ /*window_id*/) => {
                if let Some(model) = model.as_ref() {
                    if let Some(mut frame) = app.window.swap_chain.next_frame(&app.device, &mut staging) {
                        // If the user specified an app view function, use it.
                        // TODO: mandatory?
                        if let Some(view_fn) = view_fn {
                            (view_fn)(&app, &model, &mut frame);
                        }

                        frame.submit(&app.queue);
                    }
                }
            }

            // Poll devices.
            winit::event::Event::RedrawEventsCleared => {
                // TODO: This seems to cause some glitching and slows down macOS drastically.
                // While not necessary, this would be nice to have to automatically process async
                // read/write callbacks submitted by users who aren't aware that they need to poll
                // their devices in order to make them do work. Perhaps as a workaround we could
                // only poll devices that aren't already associated with a window?
                app.device.poll(wgpu::Maintain::Poll);
            }

            // Ignore wake-up events for now. Currently, these can only be triggered via the app proxy.
            winit::event::Event::NewEvents(_) => {}

            // Ignore any other events
            _ => {}
        }

        // We must re-build the swap chain if the window was resized.
        if let winit::event::Event::WindowEvent { ref mut event, .. } = event {
            let window = &mut app.window;
            match event {
                winit::event::WindowEvent::Resized(size) => {
                    window.size = *size;
                    window.rebuild_swap_chain(&app.device, *size);
                }

                winit::event::WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size: size,
                } => {
                    window.size = **size;
                    window.scale_factor = *scale_factor;
                    window.rebuild_swap_chain(&app.device, **size);
                }

                _ => (),
            }
        }

        // Process the event with the users functions and see if we need to exit.
        // if let Some(model) = model.as_mut() {
        //     exit |= process_window_event::<M, E>(&mut app, model, event_fn, &event);
        // }

        // If we need to exit, call the user's function and update control flow.
        use winit::event_loop::ControlFlow;
        if exit {
            if let Some(model) = model.take() {
                if let Some(exit_fn) = exit_fn {
                    exit_fn(&app, model);
                }
            }

            *control_flow = ControlFlow::Exit;
        } else {
            *control_flow = ControlFlow::Poll;
        }
    });
}

// Event handling boilerplate shared between the loop modes.
//
// 1. Checks for exit on escape.
// 2. Emits event via `event_fn`.
// 3. Returns whether or not we should break from the loop.
/*
fn process_event<'a, M, E>(
    app: &mut App,
    model: &mut M,
    event_fn: Option<EventFn<M, E>>,
    winit_event: &winit::event::Event<'a, ()>,
) -> bool
where
    M: 'static,
    E: LoopEvent,
{
    // Inspect the event to see if it would require closing the App.
    let mut exit_on_escape = false;
    let mut removed_window = None;
    if let winit::event::Event::WindowEvent {
        window_id,
        ref event,
    } = *winit_event
    {
        // If we should exit the app on escape, check for the escape key.
        if app.exit_on_escape() {
            if let winit::event::WindowEvent::KeyboardInput { input, .. } = *event {
                if let Some(Key::Escape) = input.virtual_keycode {
                    exit_on_escape = true;
                }
            }
        }

        // When a window has been closed, this function is called to remove any state associated
        // with that window so that the state doesn't leak.
        //
        // Returns the `Window` that was removed.
        fn remove_related_window_state(app: &App, window_id: &window::Id) -> Option<Window> {
            app.draw_state.renderers.borrow_mut().remove(window_id);
            app.windows.borrow_mut().remove(window_id)
        }

        if let winit::event::WindowEvent::Destroyed = *event {
            removed_window = remove_related_window_state(app, &window_id);
        // TODO: We should allow the user to handle this case. E.g. allow for doing things like
        // "would you like to save". We currently do this with the app exit function, but maybe a
        // window `close` function would be useful?
        } else if let winit::event::WindowEvent::CloseRequested = *event {
            removed_window = remove_related_window_state(app, &window_id);
        } else {
            // Get the size of the window for translating coords and dimensions.
            let (win_w, win_h, scale_factor) = match app.window(window_id) {
                Some(win) => {
                    // If we should toggle fullscreen for this window, do so.
                    if app.fullscreen_on_shortcut() {
                        if should_toggle_fullscreen(event, &app.keys.mods) {
                            if win.is_fullscreen() {
                                win.set_fullscreen(false);
                            } else {
                                win.set_fullscreen(true);
                            }
                        }
                    }

                    let sf = win.tracked_state.scale_factor;
                    let (w, h) = win.tracked_state.physical_size.to_logical::<f32>(sf).into();
                    (w, h, sf)
                }
                None => (0.0, 0.0, 1.0),
            };

            // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
            let tx = |x: geom::scalar::Default| x - win_w as geom::scalar::Default / 2.0;
            let ty = |y: geom::scalar::Default| -(y - win_h as geom::scalar::Default / 2.0);

            // If the window ID has changed, ensure the dimensions are up to date.
            if *app.focused_window.borrow() != Some(window_id) {
                if app.window(window_id).is_some() {
                    *app.focused_window.borrow_mut() = Some(window_id);
                }
            }

            // Check for events that would update either mouse, keyboard or window state.
            match *event {
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    let (x, y) = position.to_logical::<f32>(scale_factor).into();
                    let x = tx(x);
                    let y = ty(y);
                    app.mouse.x = x;
                    app.mouse.y = y;
                    app.mouse.window = Some(window_id);
                }

                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    match state {
                        event::ElementState::Pressed => {
                            let p = app.mouse.position();
                            app.mouse.buttons.press(button, p);
                        }
                        event::ElementState::Released => {
                            app.mouse.buttons.release(button);
                        }
                    }
                    app.mouse.window = Some(window_id);
                }

                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        match input.state {
                            event::ElementState::Pressed => {
                                app.keys.down.keys.insert(key);
                            }
                            event::ElementState::Released => {
                                app.keys.down.keys.remove(&key);
                            }
                        }
                    }
                }

                _ => (),
            }

            // See if the event could be interpreted as a `ui::Input`. If so, submit it to the
            // `Ui`s associated with this window.
            if let Some(window) = app.windows.borrow().get(&window_id) {
                if let Some(input) = ui::winit_window_event_to_input(event, window) {
                    if let Some(handles) = app.ui.windows.borrow().get(&window_id) {
                        for handle in handles {
                            if let Some(ref tx) = handle.input_tx {
                                tx.try_send(input.clone()).ok();
                            }
                        }
                    }
                }
            }
        }
    }

    // Update the modifier keys within the app if necessary.
    if let winit::event::Event::WindowEvent { event, .. } = winit_event {
        if let winit::event::WindowEvent::ModifiersChanged(new_mods) = event {
            app.keys.mods = new_mods.clone();
        }
    }

    // If the user provided an event function and winit::event::Event could be interpreted as some event
    // `E`, use it to update the model.
    if let Some(event_fn) = event_fn {
        if let Some(event) = E::from_winit_event(winit_event, app) {
            event_fn(&app, model, event);
        }
    }

    // If the event was a window event, and the user specified an event function for this window,
    // call it.
    if let winit::event::Event::WindowEvent {
        window_id,
        ref event,
    } = *winit_event
    {
        // Raw window events.
        if let Some(raw_window_event_fn) = {
            let windows = app.windows.borrow();
            windows
                .get(&window_id)
                .and_then(|w| w.user_functions.raw_event.clone())
                .or_else(|| {
                    removed_window
                        .as_ref()
                        .and_then(|w| w.user_functions.raw_event.clone())
                })
        } {
            let raw_window_event_fn = raw_window_event_fn
                .to_fn_ptr::<M>()
                .expect("unexpected model argument given to window event function");
            (*raw_window_event_fn)(&app, model, event);
        }

        let (win_w, win_h, scale_factor) = {
            let windows = app.windows.borrow();
            windows
                .get(&window_id)
                .map(|w| {
                    let sf = w.tracked_state.scale_factor;
                    let (w, h) = w.tracked_state.physical_size.to_logical::<f64>(sf).into();
                    (w, h, sf)
                })
                .unwrap_or((0.0, 0.0, 1.0))
        };

        // If the event can be represented by a simplified nannou event, check for relevant user
        // functions to be called.
        if let Some(simple) =
            event::WindowEvent::from_winit_window_event(event, win_w, win_h, scale_factor)
        {
            // Nannou window events.
            if let Some(window_event_fn) = {
                let windows = app.windows.borrow();
                windows
                    .get(&window_id)
                    .and_then(|w| w.user_functions.event.clone())
                    .or_else(|| {
                        removed_window
                            .as_ref()
                            .and_then(|w| w.user_functions.event.clone())
                    })
            } {
                let window_event_fn = window_event_fn
                    .to_fn_ptr::<M>()
                    .expect("unexpected model argument given to window event function");
                (*window_event_fn)(&app, model, simple.clone());
            }

            // A macro to simplify calling event-specific user functions.
            macro_rules! call_user_function {
                ($fn_name:ident $(,$arg:expr)*) => {{
                    if let Some(event_fn) = {
                        let windows = app.windows.borrow();
                        windows
                            .get(&window_id)
                            .and_then(|w| w.user_functions.$fn_name.clone())
                            .or_else(|| {
                                removed_window
                                    .as_ref()
                                    .and_then(|w| w.user_functions.$fn_name.clone())
                            })
                    } {
                        let event_fn = event_fn
                            .to_fn_ptr::<M>()
                            .unwrap_or_else(|| {
                                panic!(
                                    "unexpected model argument given to {} function",
                                    stringify!($fn_name),
                                );
                            });
                        (*event_fn)(&app, model, $($arg),*);
                    }
                }};
            }

            // Check for more specific event functions.
            match simple {
                event::WindowEvent::KeyPressed(key) => call_user_function!(key_pressed, key),
                event::WindowEvent::KeyReleased(key) => call_user_function!(key_released, key),
                event::WindowEvent::MouseMoved(pos) => call_user_function!(mouse_moved, pos),
                event::WindowEvent::MousePressed(button) => {
                    call_user_function!(mouse_pressed, button)
                }
                event::WindowEvent::MouseReleased(button) => {
                    call_user_function!(mouse_released, button)
                }
                event::WindowEvent::MouseEntered => call_user_function!(mouse_entered),
                event::WindowEvent::MouseExited => call_user_function!(mouse_exited),
                event::WindowEvent::MouseWheel(amount, phase) => {
                    call_user_function!(mouse_wheel, amount, phase)
                }
                event::WindowEvent::Moved(pos) => call_user_function!(moved, pos),
                event::WindowEvent::Resized(size) => call_user_function!(resized, size),
                event::WindowEvent::Touch(touch) => call_user_function!(touch, touch),
                event::WindowEvent::TouchPressure(pressure) => {
                    call_user_function!(touchpad_pressure, pressure)
                }
                event::WindowEvent::HoveredFile(path) => call_user_function!(hovered_file, path),
                event::WindowEvent::HoveredFileCancelled => {
                    call_user_function!(hovered_file_cancelled)
                }
                event::WindowEvent::DroppedFile(path) => call_user_function!(dropped_file, path),
                event::WindowEvent::Focused => call_user_function!(focused),
                event::WindowEvent::Unfocused => call_user_function!(unfocused),
                event::WindowEvent::Closed => call_user_function!(closed),
            }
        }
    }

    // If the loop was destroyed, we'll need to exit.
    let loop_destroyed = match winit_event {
        winit::event::Event::LoopDestroyed => true,
        _ => false,
    };

    // If any exist conditions were triggered, indicate so.
    let exit = if loop_destroyed || exit_on_escape || app.windows.borrow().is_empty() {
        true
    } else {
        false
    };

    exit
}
*/
