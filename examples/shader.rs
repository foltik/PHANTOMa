#![feature(exclusive_range_pattern)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use phantoma::prelude::*;
use phantoma::gfx::pass::{FilterPass, SynthPass};
use phantoma::gfx::uniform::Uniform;

use rand::prelude::*;

fn main() {
    phantoma::app::run(model, input, update, view);
}

pub struct Model {
    time: Uniform<f32>,
    circle: SynthPass,
    glitch: FilterPass,
}

async fn model(app: &App) -> Model {
    let device = &app.device;

    let time = Uniform::new(device, "time", Some(&0.0));

    let circle = SynthPass::new(device, "shader", "circle.frag.spv", Some(&time));
    let glitch = FilterPass::new(device, "glitch", "simpleglitch.frag.spv", Some(&time));

    Model {
        time,
        circle,
        glitch
    }
}

async fn input(app: &App, m: &mut Model, state: KeyState, key: Key) {}
async fn update(app: &App, m: &mut Model, dt: f32) {}

fn view(app: &App, m: &mut Model, frame: &mut Frame, view: &wgpu::RawTextureView) {
    m.time.upload(frame, &app.t);
    m.circle.encode(frame, m.glitch.view(0));
    m.glitch.encode(frame, view);
}