use nannou::prelude::*;
use lib::gfx::{Drawer, Effect, Present};

fn main() {
    nannou::app(model).update(update).view(view).run();
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Uniform {
    t: f32,
}

struct Model {
    drawer: Drawer,
    glitch: Effect<Uniform>,
    discolor: Effect<Uniform>,
    present: Present,
    uniform: Uniform,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .size(1920, 1080)
        .title("PHANTOMa")
        .view(view)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();

    let device = window.swap_chain_device();
    let samples = window.msaa_samples();

    let drawer = Drawer::new(device, samples);
    let glitch = Effect::new(device, "../resources/shaders/glitch.frag.spv");
    let discolor = Effect::new(device, "../resources/shaders/discolor.frag.spv");
    let present = Present::new(device, window.msaa_samples());

    Model {
        drawer,
        glitch,
        discolor,
        present,
        uniform: Uniform { t: 0.0 },
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    model.uniform.t = app.time;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(BLACK);
    draw.ellipse()
        .no_fill()
        .stroke_color(WHITE)
        .stroke_weight(10.0)
        .x_y(0.0, 0.0)
        .radius(500.0);

    let window = app.main_window();
    let device = window.swap_chain_device();
    let mut encoder = frame.command_encoder();

    model.drawer.encode(device, &mut encoder, model.glitch.view(), &draw);

    model.glitch.update(&device, &mut encoder, &model.uniform);
    model.glitch.encode(&mut encoder, model.discolor.view());

    model.discolor.update(&device, &mut encoder, &model.uniform);
    model.discolor.encode(&mut encoder, model.present.view());

    model.present.encode(&mut encoder, &frame);
}
