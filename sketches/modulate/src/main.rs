use nannou::prelude::*;
use modulator::{ModulatorEnv, sources::*};

fn main() {
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .run();
}

struct Model {
    mods: ModulatorEnv<f32>
}

fn model(_app: &App) -> Model {
    let mut mods = ModulatorEnv::new();

    let mut follower = Box::new(ScalarGoalFollower::new(Box::new(Newtonian::new(
        [2.0, 12.0],
        [4.0, 24.0],
        [4.0, 24.0],
        0.0,
    ))));
    follower.regions.push([-3.0, 3.0]);

    mods.take("follower", follower);

    Model {
        mods
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    model.mods.advance(update.since_last.as_micros() as u64);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let mods = &model.mods;

    draw.background()
        .color(BLACK);

    draw.tri()
        .w_h(100.0, 100.0)
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(2.0)
        .rotate(mods.value("follower"));

    draw.to_frame(app, &frame).unwrap();
}
