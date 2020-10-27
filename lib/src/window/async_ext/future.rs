use super::{EventAsync, SharedState, WaitCanceledCause};
use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};
use winit::{
    event::{Event, StartCause},
    event_loop::ControlFlow,
    window::WindowId,
};

#[must_use]
pub struct WaitFuture<'a, E: 'static> {
    pub(crate) shared_state: &'a RefCell<SharedState<E>>,
}

#[must_use]
pub struct WaitUntilFuture<'a, E: 'static> {
    pub(crate) timeout: Instant,
    pub(crate) shared_state: &'a RefCell<SharedState<E>>,
}

#[must_use]
pub(crate) struct EventReceiverBuilder<'a, E: 'static> {
    pub(crate) shared_state: &'a RefCell<SharedState<E>>,
}

pub struct EventReceiver<'a, E: 'static> {
    pub(crate) shared_state: &'a RefCell<SharedState<E>>,
}

#[must_use]
pub struct PollFuture<'a, E: 'static> {
    pub(crate) shared_state: &'a RefCell<SharedState<E>>,
    pub(crate) sealed: bool,
}

#[must_use]
pub(crate) struct RedrawRequestReceiverBuilder<'a, E: 'static> {
    pub(crate) shared_state: &'a RefCell<SharedState<E>>,
}

pub struct RedrawRequestReceiver<'a, E: 'static> {
    pub(crate) shared_state: &'a RefCell<SharedState<E>>,
}

#[must_use]
pub struct RedrawRequestFuture<'a, E: 'static> {
    pub(crate) shared_state: &'a RefCell<SharedState<E>>,
    pub(crate) sealed: bool,
}

impl<E> Future for WaitFuture<'_, E> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<()> {
        let mut shared_state = self.shared_state.borrow_mut();
        match shared_state.next_event {
            Some(Event::NewEvents { .. }) => {
                unsafe { *shared_state.control_flow.unwrap().as_mut() = ControlFlow::Poll };
                Poll::Ready(())
            }
            Some(Event::RedrawEventsCleared) => {
                unsafe { *shared_state.control_flow.unwrap().as_mut() = ControlFlow::Wait };
                shared_state.next_event = None;
                Poll::Pending
            }
            Some(Event::WindowEvent { .. })
            | Some(Event::DeviceEvent { .. })
            | Some(Event::UserEvent { .. })
            | Some(Event::MainEventsCleared)
            | Some(Event::RedrawRequested { .. }) => {
                shared_state.next_event = None;
                Poll::Pending
            }
            Some(Event::Suspended) | Some(Event::Resumed) => unimplemented!(),
            Some(Event::LoopDestroyed) => unreachable!(),
            None => Poll::Pending,
        }
    }
}

impl<E> Future for WaitUntilFuture<'_, E> {
    type Output = WaitCanceledCause;

    fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<WaitCanceledCause> {
        let mut shared_state = self.shared_state.borrow_mut();
        match shared_state.next_event {
            Some(Event::NewEvents(cause)) => {
                unsafe { *shared_state.control_flow.unwrap().as_mut() = ControlFlow::Poll };
                Poll::Ready(match cause {
                    StartCause::ResumeTimeReached { .. } => WaitCanceledCause::ResumeTimeReached,
                    StartCause::WaitCancelled { .. } | StartCause::Poll | StartCause::Init => {
                        WaitCanceledCause::EventsReceived
                    }
                })
            }
            Some(Event::RedrawEventsCleared) => {
                unsafe {
                    *shared_state.control_flow.unwrap().as_mut() =
                        ControlFlow::WaitUntil(self.timeout)
                };
                shared_state.next_event = None;
                Poll::Pending
            }
            Some(Event::WindowEvent { .. })
            | Some(Event::DeviceEvent { .. })
            | Some(Event::UserEvent { .. })
            | Some(Event::MainEventsCleared)
            | Some(Event::RedrawRequested { .. }) => {
                shared_state.next_event = None;
                Poll::Pending
            }
            Some(Event::LoopDestroyed) => unreachable!(),
            Some(Event::Suspended) | Some(Event::Resumed) => unimplemented!(),
            None => Poll::Pending,
        }
    }
}

impl<'el, E> Future for EventReceiverBuilder<'el, E> {
    type Output = EventReceiver<'el, E>;
    fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<EventReceiver<'el, E>> {
        let mut shared_state = self.shared_state.borrow_mut();
        match shared_state.next_event {
            Some(Event::RedrawRequested { .. })
            | Some(Event::RedrawEventsCleared)
            | Some(Event::MainEventsCleared) => {
                shared_state.next_event = None;
                Poll::Pending
            }
            Some(Event::NewEvents(_)) => {
                shared_state.next_event = None;
                Poll::Ready(EventReceiver {
                    shared_state: self.shared_state,
                })
            }
            Some(Event::WindowEvent { .. })
            | Some(Event::DeviceEvent { .. })
            | Some(Event::UserEvent { .. })
            | Some(Event::LoopDestroyed) => unreachable!(),
            Some(Event::Suspended) | Some(Event::Resumed) => unimplemented!(),
            None => Poll::Pending,
        }
    }
}

impl<'el, E> EventReceiver<'el, E> {
    pub fn next(&mut self) -> PollFuture<'_, E> {
        PollFuture {
            shared_state: &self.shared_state,
            sealed: false,
        }
    }

    pub fn redraw_requests(self) -> impl Future<Output = RedrawRequestReceiver<'el, E>> {
        RedrawRequestReceiverBuilder {
            shared_state: &self.shared_state,
        }
    }
}

impl<E> Future for PollFuture<'_, E> {
    type Output = Option<EventAsync>;

    fn poll(mut self: Pin<&mut Self>, _: &mut Context) -> Poll<Option<EventAsync>> {
        if self.sealed {
            return Poll::Ready(None);
        }

        let mut shared_state = self.shared_state.borrow_mut();
        match shared_state.next_event.take() {
            Some(Event::WindowEvent { window_id: _, event }) => {
                Poll::Ready(Some(EventAsync::WindowEvent(event)))
            }
            Some(Event::DeviceEvent { device_id: _, event }) => {
                Poll::Ready(Some(EventAsync::DeviceEvent(event)))
            }
            Some(Event::UserEvent(_)) => Poll::Ready(Some(EventAsync::UserEvent)),
            Some(Event::MainEventsCleared) => {
                self.sealed = true;
                Poll::Ready(None)
            }
            event @ Some(Event::RedrawRequested { .. })
            | event @ Some(Event::RedrawEventsCleared) => {
                shared_state.next_event = event;
                Poll::Ready(None)
            }
            Some(Event::NewEvents(_)) | Some(Event::LoopDestroyed) => unreachable!(),
            Some(Event::Suspended) | Some(Event::Resumed) => unimplemented!(),
            None => Poll::Pending,
        }
    }
}

impl<'el, E> Future for RedrawRequestReceiverBuilder<'el, E> {
    type Output = RedrawRequestReceiver<'el, E>;
    fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<RedrawRequestReceiver<'el, E>> {
        let mut shared_state = self.shared_state.borrow_mut();
        match shared_state.next_event {
            Some(Event::RedrawRequested { .. }) | Some(Event::RedrawEventsCleared) => {
                Poll::Ready(RedrawRequestReceiver {
                    shared_state: self.shared_state,
                })
            }
            Some(Event::MainEventsCleared)
            | Some(Event::WindowEvent { .. })
            | Some(Event::DeviceEvent { .. })
            | Some(Event::UserEvent { .. }) => {
                shared_state.next_event = None;
                Poll::Pending
            }
            Some(Event::NewEvents { .. }) | Some(Event::LoopDestroyed) => unreachable!(),
            Some(Event::Suspended) | Some(Event::Resumed) => unimplemented!(),
            None => Poll::Pending,
        }
    }
}

impl<'el, E> RedrawRequestReceiver<'el, E> {
    pub fn next(&mut self) -> RedrawRequestFuture<'_, E> {
        RedrawRequestFuture {
            shared_state: &self.shared_state,
            sealed: false,
        }
    }
}

impl<'el, E> Future for RedrawRequestFuture<'el, E> {
    type Output = Option<WindowId>;

    fn poll(mut self: Pin<&mut Self>, _: &mut Context) -> Poll<Option<WindowId>> {
        if self.sealed {
            return Poll::Ready(None);
        }

        let mut shared_state = self.shared_state.borrow_mut();
        match shared_state.next_event {
            Some(Event::RedrawRequested(window_id)) => {
                shared_state.next_event = None;
                Poll::Ready(Some(window_id))
            },
            Some(Event::RedrawEventsCleared) => {
                self.sealed = true;
                Poll::Ready(None)
            },
            Some(Event::WindowEvent{..}) |
            Some(Event::DeviceEvent{..}) |
            Some(Event::UserEvent{..}) |
            Some(Event::MainEventsCleared) => {
                shared_state.next_event = None;
                Poll::Pending
            },

            Some(Event::NewEvents{..}) |
            // TODO: WTF? Why do we get LoopDestroyed, and why does it work if we ignore it???
            // Some(Event::LoopDestroyed) => unreachable!(),
            Some(Event::LoopDestroyed) => Poll::Pending,
            Some(Event::Suspended) |
            Some(Event::Resumed) => unimplemented!(),
            None => Poll::Pending
        }
    }
}
