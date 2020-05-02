use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run();
}

fn view(app: &App, frame: Frame) {
    let draw = app.draw();

    let poly = |n| {
        (0..=n)
            .map(move |i| {
                let t = i as f32 / n as f32;
                let n = n as f32;
                let ofs = (TAU + (n - 2.0) * TAU / n) / 4.0;
                pt2((TAU * t + ofs).cos(), (TAU * t + ofs).sin())
            })
    };

    draw.background().color(BLACK);

    let n: usize = app.time.floor() as usize % 5 + 3;
    let fr = app.time.fract();

    let pts = poly(n)
        .zip(poly(n + 1).take(n + 1))
        .map(|(a, b)| a * (1.0 - fr) + b * fr)
        .map(|p| p * 100.0);

    draw.polygon()
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0)
        .points(pts);

    draw.to_frame(app, &frame).unwrap();
}
