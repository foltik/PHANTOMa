use lib::prelude::*;
use lib::audio::{self, Audio};
use lib::gfx::uniform::UniformStorage;
use lib::gfx::pass::{SynthPass, FilterPass};

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
    t: f32,
}

struct Model {
    t: f32,
    audio: Box<dyn Audio>,
    uniform: UniformStorage<U>,
    synth: SynthPass,
    synth2: SynthPass,
    composite: FilterPass,
    filter: FilterPass,
}

fn model(app: &App) -> Model {
    let device = &app.device;

    let uniform = UniformStorage::new(&app.device, "test", U {
        r: 1.0,
        c: Vector4::new(1.0, 1.0, 1.0, 0.0),
        w: 1.0,
        t: 0.0,
    });

    let synth = SynthPass::new(device, "synth", "../resources/shaders/tcircle.frag.spv", Some(uniform.as_ref()));
    let synth2 = SynthPass::new::<()>(device, "synth2", "../resources/shaders/tcirclesmall.frag.spv", None);

    let composite = FilterPass::new_composite::<()>(device, "composite", 2, None, None);

    let filter = FilterPass::new(device, "filter", "../resources/shaders/tfilter.frag.spv", Some(uniform.as_ref()));

    Model {
        t: 0.0,
        audio: Box::new(audio::init()),
        uniform,
        synth,
        synth2,
        composite,
        filter,
    }
}

fn update(_app: &App, model: &mut Model, dt: f32) {
    model.t += dt;

    let freq = 1.0;
    let r = (freq * model.t + 0.0).sin() * 0.5 + 0.5;
    let g = (freq * model.t + 2.0).sin() * 0.5 + 0.5;
    let b = (freq * model.t + 4.0).sin() * 0.5 + 0.5;

    model.audio.update();

    let u = &mut model.uniform;
    u.c = Vector4::new(r, g, b, 0.0);
    u.r = 0.5 * model.t.sin() + 0.5;
    u.w = 0.5 * (model.t * 3.0).sin() + 0.5;
    u.t = model.t;
}

fn view(_app: &App, model: &Model, frame: &mut Frame) {
    model.uniform.update(frame);

    let encoder = &mut frame.encoder;

    model.synth.encode(encoder, &model.composite.view(0));
    model.synth2.encode(encoder, &model.composite.view(1));

    model.composite.encode(encoder, &model.filter.view(0));

    model.filter.encode(encoder, &frame.view);
}
