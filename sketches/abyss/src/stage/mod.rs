use lib::prelude::*;
use lib::midi2::device::launchpad_x;
use lib::midi2::device::worlde_easycontrol9;

use async_trait::async_trait;

use crate::{Model, StencilPass};

// mod template; use template::Template;

mod boot; use boot::Boot;
mod wave; use wave::Wave;
mod arcane; use arcane::Arcane;
mod cyber; use cyber::Cyber;
mod neon; use neon::Neon;
mod space; use space::Space;
mod tunnel; use tunnel::Tunnel;

// mod core;
// use self::core::Core;

#[async_trait]
pub trait Render {
    async fn init(&mut self, app: &App, m: &mut Model);

    async fn key(&mut self, app: &App, m: &mut Model, key: Key) {}
    async fn ctrl(&mut self, app: &App, m: &mut Model, input: worlde_easycontrol9::Input) {}
    async fn pad(&mut self, app: &App, m: &mut Model, input: launchpad_x::Input) {}
    async fn update(&mut self, app: &App, m: &mut Model, dt: f32);

    fn view(&mut self, frame: &mut Frame, stencil: &StencilPass);
}

enum Renderer {
    Boot(Boot),
    Wave(Wave),
    Tunnel(Tunnel),
    Arcane(Arcane),
    Cyber(Cyber),
    Neon(Neon),
    Space(Space),
}

pub struct Stage {
    i: usize,
    init: bool,
    renderers: Vec<Renderer>,
}

impl Stage {
    pub fn new(device: &wgpu::Device, stencil: &StencilPass) -> Self {
        Self {
            i: 0,
            init: false,
            renderers: vec![
                Renderer::Boot(Boot::new(device, stencil)),
            ]
        }
    }


    fn current(&mut self) -> &mut Renderer {
        &mut self.renderers[self.i]
    }

    pub async fn go(mut self, app: &App, m: &mut Model, i: i32) -> Self {
        self.i = i.rem_euclid(self.renderers.len() as i32) as usize;
        self.current().init(app, m).await;
        self
    }

    pub async fn go_next(self, app: &App, m: &mut Model) -> Self {
        let i = self.i as i32 + 1;
        self.go(app, m, i).await
    }

    pub async fn go_prev(self, app: &App, m: &mut Model) -> Self {
        let i = self.i as i32 - 1;
        self.go(app, m, i).await
    }


    pub async fn key(mut self, app: &App, m: &mut Model, key: Key) -> Self {
        self.current().key(app, m, key).await;
        self
    }

    pub async fn ctrl(mut self, app: &App, m: &mut Model, input: worlde_easycontrol9::Input) -> Self {
        self.current().ctrl(app, m, input).await;
        self
    }

    pub async fn pad(mut self, app: &App, m: &mut Model, input: launchpad_x::Input) -> Self {
        self.current().pad(app, m, input).await;
        self
    }


    pub async fn update(mut self, app: &App, m: &mut Model, dt: f32) -> Self {
        if !self.init {
            self.current().init(app, m).await;
            self.init = true;
        }
        self.current().update(app, m, dt).await;
        self
    }

    pub fn encode(mut self, frame: &mut Frame, stencil: &StencilPass) -> Self {
        self.current().view(frame, stencil);
        self
    }
}

// can't enum_dispatch since we need async_trait
#[async_trait]
impl Render for Renderer {
    async fn init(&mut self, app: &App, m: &mut Model) {
        match self {
            Renderer::Boot(ref mut r)   => r.init(app, m).await,
            Renderer::Wave(ref mut r)   => r.init(app, m).await,
            Renderer::Tunnel(ref mut r) => r.init(app, m).await,
            Renderer::Arcane(ref mut r) => r.init(app, m).await,
            Renderer::Cyber(ref mut r)  => r.init(app, m).await,
            Renderer::Neon(ref mut r)   => r.init(app, m).await,
            Renderer::Space(ref mut r)  => r.init(app, m).await,
        }
    }

    async fn key(&mut self, app: &App, m: &mut Model, key: Key) {
        match self {
            Renderer::Boot(ref mut r)   => r.key(app, m, key).await,
            Renderer::Wave(ref mut r)   => r.key(app, m, key).await,
            Renderer::Tunnel(ref mut r) => r.key(app, m, key).await,
            Renderer::Arcane(ref mut r) => r.key(app, m, key).await,
            Renderer::Cyber(ref mut r)  => r.key(app, m, key).await,
            Renderer::Neon(ref mut r)   => r.key(app, m, key).await,
            Renderer::Space(ref mut r)  => r.key(app, m, key).await,
        }
    }
    async fn ctrl(&mut self, app: &App, m: &mut Model, input: worlde_easycontrol9::Input) {
        match self {
            Renderer::Boot(ref mut r)   => r.ctrl(app, m, input).await,
            Renderer::Wave(ref mut r)   => r.ctrl(app, m, input).await,
            Renderer::Tunnel(ref mut r) => r.ctrl(app, m, input).await,
            Renderer::Arcane(ref mut r) => r.ctrl(app, m, input).await,
            Renderer::Cyber(ref mut r)  => r.ctrl(app, m, input).await,
            Renderer::Neon(ref mut r)   => r.ctrl(app, m, input).await,
            Renderer::Space(ref mut r)  => r.ctrl(app, m, input).await,
        }
    }
    async fn pad(&mut self, app: &App, m: &mut Model, input: launchpad_x::Input) {
        match self {
            Renderer::Boot(ref mut r)   => r.pad(app, m, input).await,
            Renderer::Wave(ref mut r)   => r.pad(app, m, input).await,
            Renderer::Tunnel(ref mut r) => r.pad(app, m, input).await,
            Renderer::Arcane(ref mut r) => r.pad(app, m, input).await,
            Renderer::Cyber(ref mut r)  => r.pad(app, m, input).await,
            Renderer::Neon(ref mut r)   => r.pad(app, m, input).await,
            Renderer::Space(ref mut r)  => r.pad(app, m, input).await,
        }
    }
    async fn update(&mut self, app: &App, m: &mut Model, dt: f32) {
        match self {
            Renderer::Boot(ref mut r)   => r.update(app, m, dt).await,
            Renderer::Wave(ref mut r)   => r.update(app, m, dt).await,
            Renderer::Tunnel(ref mut r) => r.update(app, m, dt).await,
            Renderer::Arcane(ref mut r) => r.update(app, m, dt).await,
            Renderer::Cyber(ref mut r)  => r.update(app, m, dt).await,
            Renderer::Neon(ref mut r)   => r.update(app, m, dt).await,
            Renderer::Space(ref mut r)  => r.update(app, m, dt).await,
        }
    }

    fn view(&mut self, frame: &mut Frame, stencil: &StencilPass) {
        match self {
            Renderer::Boot(ref mut r)   => r.view(frame, stencil),
            Renderer::Wave(ref mut r)   => r.view(frame, stencil),
            Renderer::Tunnel(ref mut r) => r.view(frame, stencil),
            Renderer::Arcane(ref mut r) => r.view(frame, stencil),
            Renderer::Cyber(ref mut r)  => r.view(frame, stencil),
            Renderer::Neon(ref mut r)   => r.view(frame, stencil),
            Renderer::Space(ref mut r)  => r.view(frame, stencil),
        }
    }
}
