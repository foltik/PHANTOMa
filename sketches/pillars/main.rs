use nannou::prelude::*;
use lib::{BeatDecay, audio, audio::Audio};

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

fn pillars(draw: &Draw, s: &Rect, t: f32) {
    let f = |h: f32, var: f32, t: f32| h + var + (var / 2.0) * (t * 10.0).sin();

    let heights = vec![
        f(620.0, 200.0, t * 0.5 - PI / 1.8),
        f(520.0, 100.0, t * 0.55 - PI / 2.0),
        f(400.0, 80.0, t * 0.6 - PI / 2.8),
        f(350.0, 70.0, t * 0.7 - PI / 3.8),
        f(280.0, 70.0, t * 0.75 - PI / 4.0),
        f(200.0, 70.0, t * 0.75 - PI / 3.75),
        f(150.0, 70.0, t * 0.78 - PI / 3.0),
        f(120.0, 70.0, t * 0.8 - PI / 2.5),
        f(120.0, 70.0, t * 0.81 + PI / 2.0),
        f(150.0, 70.0, t * 0.79 + PI / 4.0),
        f(200.0, 70.0, t * 0.74 + PI / 3.75),
        f(280.0, 70.0, t * 0.74 + PI / 2.5),
        f(350.0, 70.0, t * 0.71 + PI / 2.8),
        f(400.0, 70.0, t * 0.61 + PI / 2.9),
        f(520.0, 100.0, t * 0.54 + PI / 3.2),
        f(620.0, 200.0, t * 0.52 + PI / 1.8),
    ];

    let n = heights.len() as f32;
    let w = s.w() / n;
    heights.into_iter().enumerate().for_each(|(i, h)| {
        draw.rect()
            .no_fill()
            .stroke(WHITE)
            .stroke_weight(1.0)
            .x((i as f32 / n) * s.w() - s.w() / 2.0 + 0.5 * w)
            .y(-s.h() / 2.0)
            .w_h(w, h);
    });
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let s = app.window_rect();
    let t = model.t;

    draw.background()
        .color(BLACK);

    pillars(&draw, &s, t);

    // Pillars

    draw.to_frame(app, &frame).unwrap();
}
