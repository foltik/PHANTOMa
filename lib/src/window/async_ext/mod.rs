mod future;

use std::{
    cell::RefCell,
    future::Future,
    ptr,
    rc::Rc,
    task::{Context, Poll},
    time::Instant,
};
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub struct EventLoopRunnerAsync<E: 'static> {
    shared_state: Rc<RefCell<SharedState<E>>>,
}

pub(crate) struct SharedState<E: 'static> {
    next_event: Option<Event<'static, E>>,
    control_flow: Option<ptr::NonNull<ControlFlow>>,
}

#[must_use]
pub enum WaitCanceledCause {
    ResumeTimeReached,
    EventsReceived,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EventAsync {
    WindowEvent(WindowEvent<'static>),
    DeviceEvent(DeviceEvent),
    UserEvent,
    Suspended,
    Resumed,
    // WindowEvent {
    //     window_id: WindowId,
    //     event: WindowEvent<'static>,
    // },
    // DeviceEvent {
    //     device_id: DeviceId,
    //     event: DeviceEvent,
    // },
    // UserEvent(E),
    // Suspended,
    // Resumed,
}

pub trait EventLoopAsync {
    type Event: 'static;
    fn run_async<Fn, Fu>(self, event_handler: Fn) -> !
    where
        Fn: 'static + FnOnce(EventLoopRunnerAsync<Self::Event>) -> Fu,
        Fu: Future<Output = ()>;
}

impl<E: 'static + std::fmt::Debug> EventLoopAsync for EventLoop<E> {
    type Event = E;

    fn run_async<Fn, Fu>(self, event_handler: Fn) -> !
    where
        Fn: 'static + FnOnce(EventLoopRunnerAsync<E>) -> Fu,
        Fu: Future<Output = ()>,
    {
        let shared_state = Rc::new(RefCell::new(SharedState {
            next_event: None,
            control_flow: None,
        }));
        let shared_state_clone = shared_state.clone();
        let mut future = Box::pin(async move {
            let runner = EventLoopRunnerAsync {
                shared_state: shared_state_clone,
            };
            event_handler(runner).await
        });

        let waker = futures::task::noop_waker_ref();

        self.run(move |event, _, control_flow| {
            let control_flow_ptr = control_flow as *mut ControlFlow;
            {
                let mut shared_state = shared_state.borrow_mut();
                shared_state.control_flow = ptr::NonNull::new(control_flow_ptr);
                println!("sending event..");
                shared_state.next_event = Some(
                    event
                        .to_static()
                        .expect("Couldn't make WindowEvent 'static: did the DPI change?"),
                );
            }

            if unsafe { *control_flow_ptr } != ControlFlow::Exit {
                let mut context = Context::from_waker(waker);
                match future.as_mut().poll(&mut context) {
                    Poll::Ready(()) => unsafe { *control_flow_ptr = ControlFlow::Exit },
                    Poll::Pending => {
                        unsafe { *control_flow_ptr = ControlFlow::Poll };
                        shared_state.borrow_mut().control_flow = None;
                    },
                }
            }
        });
    }
}

impl<E> EventLoopRunnerAsync<E> {
    pub fn wait(&mut self) -> future::WaitFuture<'_, E> {
        future::WaitFuture {
            shared_state: &self.shared_state,
        }
    }

    pub fn wait_until(&mut self, timeout: Instant) -> future::WaitUntilFuture<'_, E> {
        future::WaitUntilFuture {
            timeout,
            shared_state: &self.shared_state,
        }
    }

    pub fn recv_events(&mut self) -> impl '_ + Future<Output = future::EventReceiver<'_, E>> {
        future::EventReceiverBuilder {
            shared_state: &self.shared_state,
        }
    }
}