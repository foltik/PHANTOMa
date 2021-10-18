use lib::prelude::*;
use lib::midi2::device::launchpad_x;
use lib::midi2::device::worlde_easycontrol9;

use lib::time::DecayEnv;

use crate::{Model, StencilPass};
use crate::pipeline::*;

use super::Render;

pub struct Template {
    decay: DecayEnv,
}

impl Template {
    pub fn new(device: &wgpu::Device, s: &StencilPass) -> Self {
        let decay = DecayEnv::default()
            .with("beat", 250.0);

        Self {
            decay
        }
    }
}

impl Render for Template {
    fn init(&mut self, _app: &App, m: &mut Model) {

    }

    fn key(&mut self, _app: &App, m: &mut Model, key: Key) {
        match key {
            _ => {}
        }
    }

    fn ctrl(&mut self, _app: &App, m: &mut crate::Model, input: worlde_easycontrol9::Input) {
        use worlde_easycontrol9::Input;
        match input {
            _ => {}
        }
    }

    fn pad(&mut self, _app: &App, m: &mut crate::Model, input: launchpad_x::Input) {
        use launchpad_x::{*, types::*};
        use PaletteColor as Color;
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

    fn update(&mut self, _app: &App, m: &mut Model, dt: f32) {
    }

    fn view(&mut self, frame: &mut Frame, s: &StencilPass) {
    }
}