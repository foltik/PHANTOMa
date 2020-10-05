use crate::gfx::frame::Frame;
use crate::window::WindowBuilder;

use super::App;


pub type ModelFn<Model> = fn(&App) -> Model;
pub type WindowFn = fn(WindowBuilder) -> WindowBuilder;
//pub type EventFn<Model, Event> = fn(&App, &mut Model, Event);
pub type UpdateFn<Model> = fn(&App, &mut Model, f32);
pub type ViewFn<Model> = fn(&App, &Model, &mut Frame);
pub type ExitFn<Model> = fn(&App, Model);

pub struct Builder<M = ()> {
    pub(crate) model: ModelFn<M>,
    pub(crate) window: Option<WindowFn>,
    //event: Option<EventFn<M, E>>,
    pub(crate) update: Option<UpdateFn<M>>,
    pub(crate) view: Option<ViewFn<M>>,
    pub(crate) exit: Option<ExitFn<M>>,
}

impl<M: 'static> Builder<M> {
    pub fn new(model: ModelFn<M>) -> Self {
        Builder {
            model,
            window: None,
            //event: None,
            update: None,
            view: None,
            exit: None,
        }
    }

    pub fn window(mut self, window: WindowFn) -> Self {
        self.window = Some(window);
        self
    }

    pub fn view(mut self, view: ViewFn<M>) -> Self {
        self.view = Some(view);
        self
    }

    pub fn update(mut self, update: UpdateFn<M>) -> Self {
        self.update = Some(update);
        self
    }

    pub fn exit(mut self, exit: ExitFn<M>) -> Self {
        self.exit = Some(exit);
        self
    }

    pub fn run(self) {
        super::startup(self);
    }
}
