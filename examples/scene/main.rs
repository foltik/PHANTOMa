#![feature(exclusive_range_pattern)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use phantoma::prelude::*;
use phantoma::gfx::uniform::Uniform;
use phantoma::gfx::scene::Scene;

mod phong;
use phong::PhongPipeline;

fn main() {
    phantoma::app::run(model, input, update, view);
}

pub struct Model {
    scene: Scene,
    phong: PhongPipeline,
}

async fn model(app: &App) -> Model {
    let device = &app.device;

    let scene = phantoma::resource::read_scene(device, "core3.glb");
    // let animator = Animator::new(&scene);

    let phong = PhongPipeline::new(app, &scene);

    let depth = wgpu::util::TextureBuilder::new_depth("depth")
        .build(&app.device)
        .view()
        .build();

    Model {
        scene,
        phong,
    }
}

async fn input(app: &App, m: &mut Model, state: KeyState, key: Key) {}
async fn update(app: &App, m: &mut Model, dt: f32) {}

fn view(app: &App, m: &mut Model, frame: &mut Frame, view: &wgpu::RawTextureView) {
    m.phong.encode(frame, &m.scene,view);
}