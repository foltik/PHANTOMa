use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[derive(Clone)]
pub struct Proxy {
    proxy: winit::event_loop::EventLoopProxy<()>,
    // Indicates whether or not the events loop is currently asleep.
    //
    // This is set to `true` each time the events loop is ready to return and the `LoopMode` is
    // set to `Wait` for events.
    //
    // This value is set back to `false` each time the events loop receives any kind of event.
    is_asleep: Arc<AtomicBool>,
}

impl Proxy {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        Self {
            proxy: event_loop.create_proxy(),
            is_asleep: Arc::new(AtomicBool::new(false)),
        }
    }
}
