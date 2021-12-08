#![feature(exclusive_range_pattern)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use phantoma::prelude::*;
use phantoma::gfx::pass::{TextPass, TextPassBuilder};

use rand::prelude::*;

fn main() {
    phantoma::app::run(model, input, update, view);
}

pub struct Model {
    text: TextPass,
}

async fn model(app: &App) -> Model {
    let device = &app.device;

    let text = TextPassBuilder::default()
        .with("default", "go.ttf")
        .build(device);

    Model {
        text
    }
}

async fn input(app: &App, m: &mut Model, state: KeyState, key: Key) {
    log::info!("Keyboard: {:?} {:?}", key, state);
}

async fn update(app: &App, m: &mut Model, dt: f32) {
    m.text.draw(|d| d.at(v2(1920.0 / 3.0, 1080.0 / 3.0))
        .text("Hello, world!", |t| 
            t.scale(72.0)
            .color(v4(1.0, 1.0, 1.0, 1.0))));
}

fn view(_app: &App, m: &mut Model, frame: &mut Frame, view: &wgpu::RawTextureView) {
    m.text.encode(frame, view);
}
