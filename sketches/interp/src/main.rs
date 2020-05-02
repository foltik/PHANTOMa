use lib::{*, audio, audio::Audio};
use nannou::math::cgmath::Vector2 as CVector2;
use nannou::prelude::*;

use splines::{Interpolation, Key, Spline};

type Vector2 = CVector2<f32>;

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
    //model.t += model.audio.rms();
    model.t = _app.time;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let t = model.t;

    draw.background().color(BLACK);


    // First spline with no circular tangents on loop endpoints
    let pts1 = vec![
        Vector2::new(-100.0, 100.0),
        Vector2::new(-100.0, 300.0),
        Vector2::new(100.0, 300.0),
        Vector2::new(200.0, 200.0),
        Vector2::new(100.0, 100.0),
    ];

    let keys1 = vec![
        Key::new(0.0, pts1[0], Interpolation::default()),
        Key::new(0.0, pts1[0], Interpolation::CatmullRom),
        Key::new(1.0, pts1[1], Interpolation::CatmullRom),
        Key::new(2.0, pts1[2], Interpolation::CatmullRom),
        Key::new(3.0, pts1[3], Interpolation::CatmullRom),
        Key::new(4.0, pts1[4], Interpolation::default()),
        Key::new(4.0, pts1[0], Interpolation::default()),
    ];

    let spline1 = Spline::from_vec(keys1);

    pts1.iter().for_each(|p| {
        draw.ellipse().color(WHITE).x_y(p.x, p.y).radius(5.0);
    });

    let pd1 = 4.0;
    let t1 = pd1 - ((t % (2.0 * pd1)) - pd1).abs();
    let pos1 = spline1.sample(t1).unwrap();
    draw.ellipse().color(RED).x_y(pos1.x, pos1.y).radius(3.0);



    // Second spline with clean looping
    let pts2 = vec![
        Vector2::new(-100.0, -300.0),
        Vector2::new(-100.0, -100.0),
        Vector2::new(100.0, -100.0),
        Vector2::new(200.0, -200.0),
        Vector2::new(100.0, -300.0),
    ];

    let keys2 = vec![
        Key::new(0.0, pts2[4], Interpolation::default()),
        Key::new(0.0, pts2[0], Interpolation::CatmullRom),
        Key::new(1.0, pts2[1], Interpolation::CatmullRom),
        Key::new(2.0, pts2[2], Interpolation::CatmullRom),
        Key::new(2.707, pts2[3], Interpolation::CatmullRom),
        Key::new(3.414, pts2[4], Interpolation::CatmullRom),
        Key::new(4.414, pts2[0], Interpolation::CatmullRom),
        Key::new(4.414, pts2[1], Interpolation::default()),
        Key::new(4.414, pts2[2], Interpolation::default()),
    ];

    let spline2 = Spline::from_vec(keys2);

    pts2.iter().for_each(|p| {
        draw.ellipse().color(WHITE).x_y(p.x, p.y).radius(5.0);
    });

    let pd2 = 4.414;
    let t2 = t % pd2;
    let pos2 = spline2.sample(t2).unwrap();
    draw.ellipse().color(RED).x_y(pos2.x, pos2.y).radius(3.0);


    draw.to_frame(app, &frame).unwrap();
}
