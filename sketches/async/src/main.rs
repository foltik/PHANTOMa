use lib::prelude::*;
use lib::audio::{self, Audio};
use lib::gfx::uniform::UniformStorage;
use lib::gfx::pass::synth::SynthPass;

// use lib::gfx::wgpu;
use lib::cgmath::Vector4;

fn main() {
    lib::app(model)
        .update(update)
        .view(view)
        .run();
}

#[derive(Copy, Clone)]
struct U {
    c: Vector4<f32>,
    r: f32,
    w: f32,
}

struct Model {
    t: f32,
    audio: Box<dyn Audio>,
    uniform: UniformStorage<U>,
    pass: SynthPass,
}

fn model(app: &App) -> Model {
    let device = &app.device;

    let uniform = UniformStorage::new(&app.device, "test", U {
        r: 1.0,
        c: Vector4::new(1.0, 1.0, 1.0, 0.0),
        w: 1.0,
    });

    let pass = SynthPass::new(device, "synth", "../resources/shaders/tcircle.frag.spv", Some(&uniform.uniform));

    Model {
        t: 0.0,
        audio: Box::new(audio::init()),
        uniform,
        pass,
    }
}

fn update(_app: &App, model: &mut Model, dt: f32) {
    model.t += dt;

    let freq = 1.0;
    let r = (freq * model.t + 0.0).sin() * 0.5 + 0.5;
    let g = (freq * model.t + 2.0).sin() * 0.5 + 0.5;
    let b = (freq * model.t + 4.0).sin() * 0.5 + 0.5;

    model.audio.update();

    model.uniform.v.c = Vector4::new(r, g, b, 0.0);
    model.uniform.v.r = 0.5 * model.t.sin() + 0.5;
    model.uniform.v.w = 0.5 * (model.t * 3.0).sin() + 0.5;
}

fn view(_app: &App, model: &Model, frame: &mut Frame) {
    model.uniform.update(frame);

    let encoder = &mut frame.encoder;

    model.pass.encode(encoder, &frame.view);
}
