use nannou::prelude::*;
use lib::{*, audio, audio::Audio};

fn main() {
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .run();
}

struct Model {
    audio: Box<dyn Audio>,
    beat: BeatDecay,
    t: f32,
}

fn model(_app: &App) -> Model {
    Model {
        audio: Box::new(audio::init()),
        beat: BeatDecay::new(40.0, 120.0, 0.005, false, 250.0),
        t: 0.0,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let ms = update.since_last.as_nanos() as f32 / 1_000_000.0;

    model.audio.update();
    model.beat.update(ms, &*model.audio);
    model.t += model.audio.rms();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let t = model.t;

    draw.background()
        .color(BLACK);

    draw.ellipse()
        .color(WHITE);

    draw.to_frame(app, &frame).unwrap();
}
