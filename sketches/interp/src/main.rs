use nannou::prelude::*;
use nannou::math::cgmath::Vector2;
use lib::interp;

fn main() {
    nannou::app(model).simple_window(view).run();
}

struct Model;

fn model(_app: &App) -> Model { Model }

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    let t = app.time;

    draw.background().color(BLACK);


    // First spline with no circular tangents on loop endpoints
    let pts1 = vec![
        Vector2::new(-100.0, 100.0),
        Vector2::new(-100.0, 300.0),
        Vector2::new(100.0, 300.0),
        Vector2::new(200.0, 200.0),
        Vector2::new(100.0, 100.0),
    ];

    let spline1 = interp::catmull(&pts1, 4.0);

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
    let spline2 = interp::catmull_loop(&pts2, 10.0);

    pts2.iter().for_each(|p| {
        draw.ellipse().color(GRAY).x_y(p.x, p.y).radius(5.0);
    });

    let pd2 = 10.0;
    let t2 = t % pd2;
    let pos2 = spline2.sample(t2).unwrap();
    draw.ellipse().color(RED).x_y(pos2.x, pos2.y).radius(3.0);


    draw.to_frame(app, &frame).unwrap();
}
