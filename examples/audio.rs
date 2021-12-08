#![feature(exclusive_range_pattern)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

#[cfg(target_os = "windows")]
#[link(name = "C:/Program Files/JACK2/lib/libjack64")]
extern "C" {}

use phantoma::prelude::*;
use phantoma::gfx::pass::{FilterPass, TextPass, TextPassBuilder};

fn main() {
    phantoma::app::run(model, input, update, view);
}

pub struct Model {
    audio: Audio,
    text: TextPass,
}

async fn model(app: &App) -> Model {
    let device = &app.device;

    let audio = Audio::open().expect("Failed to open audio device!");

    let text = TextPassBuilder::default()
        .with("default", "go.ttf")
        .build(device);

    Model {
        audio,
        text
    }
}

async fn input(app: &App, m: &mut Model, state: KeyState, key: Key) {}

async fn update(app: &App, m: &mut Model, dt: f32) {
    m.audio.update();

    m.text.draw(|d| d.at(v2(1920.0 / 3.0, 1080.0 / 3.0))
        .text(&format!("RMS: {}", m.audio.rms()), |t| 
            t.scale(72.0)
            .color(v4(1.0, 1.0, 1.0, 1.0))));

    m.text.draw(|d| d.at(v2(1920.0 / 3.0, 1080.0 / 2.0))
        .text(&format!("Peak: {}", m.audio.peak()), |t| 
            t.scale(72.0)
            .color(v4(1.0, 1.0, 1.0, 1.0))));
}

fn view(_app: &App, m: &mut Model, frame: &mut Frame, view: &wgpu::RawTextureView) {
    m.text.encode(frame, view);
}
