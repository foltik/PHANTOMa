use lib::prelude::*;
use gfx::pass::{SynthPass, FilterPass};

fn main() {
    // lib::app(model)
    //     .update(update)
    //     .view(view)
    //     .run();
    // lib::app::run(model, update, view);
    lib::app::run(model, update, view);
}

#[derive(Copy, Clone)]
struct U {
    c: Vector4,
    r: f32,
    w: f32,
    t: f32,
}

struct Model {
    t: f32,
    audio: Audio,
    uniform: UniformStorage<U>,
    synth: SynthPass,
    synth2: SynthPass,
    composite: FilterPass,
    filter: FilterPass,
}

async fn model(app: &App) -> Model {
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
        audio: Audio::default(),
        uniform,
        synth,
        synth2,
        composite,
        filter,
    }
}

async fn update(_app: &App, model: &mut Model, dt: f32) {
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

async fn view(app: &mut App, model: &Model, view: &wgpu::SwapChainTextureView) {
    let frame = &mut Frame::new(app);

    model.uniform.update(frame);

    model.synth.encode(frame, &model.composite.view(0));
    model.synth2.encode(frame, &model.composite.view(1));

    model.composite.encode(frame, &model.filter.view(0));

    model.filter.encode(frame, view);

    frame.submit();
}
