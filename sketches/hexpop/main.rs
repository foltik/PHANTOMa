use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run();
}

fn rand(seed: f32) -> f32 {
    let p = pt2(seed + 10.0, seed + 3.0);
    let dt = p.perp_dot(pt2(12.9898, 78.233));
    let sn = dt % 3.14;
    2.0 * (sn.sin() * 43758.5453).fract() - 1.0
}

fn view(app: &App, frame: Frame) {
    let draw = app.draw();

    draw.background().color(BLACK);

    let res = pt2(1920.0, 1080.0) / 2.0;
    let (n, r) = (10, 20.0);

    let hex: Vec<_> = (0..6)
        .map(|i| pt2((TAU * i as f32 / 6.0).cos(), (TAU * i as f32 / 6.0).sin()))
        .collect();

    for i in 1..=n {
        let i = i as f32;
        let t = app.time * 3.0 + i * 0.2;

        let pos = pt2(rand(i * t.floor()), rand(i * t.floor() + 1.0)) * res * 0.95;

        draw.polygon()
            .no_fill()
            .stroke(WHITE)
            .stroke_weight(1.0)
            .points(hex.iter().map(|v| (*v * r * (t.fract() * PI).sin()) + pos));
    }

    draw.to_frame(app, &frame).unwrap();
}
