use lib::prelude::*;
use lib::midi2::device::launchpad_x;
use lib::midi2::device::worlde_easycontrol9;

use lib::time::DecayEnv;

use async_trait::async_trait;

use crate::{Model, StencilPass};
use crate::pipeline::*;

use super::Render;

pub struct Space {
    decay: DecayEnv,
}

impl Space {
    pub fn new(device: &wgpu::Device, s: &StencilPass) -> Self {
        let decay = DecayEnv::default()
            .with("beat", 250.0);

        Self {
            decay
        }
    }
}

#[async_trait]
impl Render for Space {
    async fn init(&mut self, _app: &App, m: &mut Model) {

    }

    async fn key(&mut self, _app: &App, m: &mut Model, key: Key) {
        match key {
            _ => {}
        }
    }

    async fn ctrl(&mut self, _app: &App, m: &mut crate::Model, input: worlde_easycontrol9::Input) {
        use worlde_easycontrol9::Input;
        match input {
            _ => {}
        }
    }

    async fn pad(&mut self, _app: &App, m: &mut crate::Model, input: launchpad_x::Input) {
        use launchpad_x::{*, types::*};
        let pad = m.pad.as_ref().unwrap();

        match input {
            Input::Press(i, _) => {
                let p = Pos::from(i);
                let Coord(x, y) = p.into();

                match (x, y) {
                    _ => {}
                }
            },
            Input::Release(i) => {
                let p = Pos::from(i);
                let Coord(x, y) = p.into();

                match (x, y) {
                    _ => {}
                }
            }
            _ => {},
        }
    }

    async fn update(&mut self, _app: &App, m: &mut Model, dt: f32) {
    }

    fn view(&mut self, frame: &mut Frame, s: &StencilPass) {
    }
}
