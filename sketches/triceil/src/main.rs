use lib::{*, audio::{self, Audio}};
use nannou::prelude::*;
use nannou::text::{font, Font};
use std::collections::HashMap;

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
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

fn triceil(draw: &Draw, fonts: &HashMap<String, Font>, s: &Rect, _t: f32) {
    let y0 = 500.0;
    let m = 1.4;
    let xs: Vec<f32> = vec![
        0.0, 180.0, -150.0, 230.0, -100.0, 60.0, -90.0, 110.0, -80.0, 100.0, -140.0, 90.0, -180.0, 230.0, -200.0
    ];

    let mut pts = Vec::with_capacity(xs.len());
    let mut p = pt2(-s.w() / 2.0, s.h() / 2.0 - y0);
    for x in xs {
        p.x += x.abs();
        p.y += m * x;
        pts.push(p);
    }

    draw.path().stroke().color(WHITE).weight(1.0).points(pts.iter().map(|p| *p));

    // Rune strip top / bot
    draw.line()
        .color(WHITE)
        .weight(1.0)
        .start(pt2(pts[2].x - 20.0, pts[2].y + 20.0 * m))
        .end(pt2(pts[2].x + 380.0, pts[2].y + 420.0 * m));
    draw.line()
        .color(WHITE)
        .weight(1.0)
        .start(pts[2])
        .end(pt2(pts[2].x + 400.0, pts[2].y + 400.0 * m));

    (0..20)
        .zip(chars(5.0))
        .for_each(|(i, s)| {
            let p = pt2(i as f32 * 50.0, pts[2].y);
            draw.text(s)
                .color(RED)
                .xy(p.rotate(m.atan()))
                .rotate(m.atan())
                .font(fonts["magi"].clone())
                .font_size(24);
        });
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let mut fonts: HashMap<String, Font> = HashMap::new();
    fonts.insert("magi".to_string(), font::from_file("../../resources/fonts/magi.ttf").unwrap());
    let s = app.window_rect();
    let t = model.t;

    draw.background().color(BLACK);

    triceil(&draw, &fonts, &s, t);

    draw.to_frame(app, &frame).unwrap();
}
