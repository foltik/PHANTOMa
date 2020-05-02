use nannou::prelude::*;
use nannou::text::{font, Font};

fn main() {
    nannou::app(model).view(view).update(update).run();
}

struct Model {
    font: Font,
    amount: i32,
    hold: bool,
}

fn model(app: &App) -> Model {
    app.new_window()
        .key_pressed(key_pressed)
        .key_released(key_released)
        .build()
        .unwrap();

    Model {
        font: font::from_file("../../resources/fonts/thief.ttf").unwrap(),
        amount: 0,
        hold: false,
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => model.amount = 100,
        Key::H => {
            model.hold = true;
            model.amount = 100;
        },
        _ => {}
    };
}

fn key_released(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::H => {
            model.hold = false;
            model.amount = 0;
        },
        _ => {}
    };
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if model.amount > 0 && !model.hold {
        model.amount -= 4;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    let primary = hsl(0.0, 0.0, (100 - model.amount) as f32 / 100.0);
    let secondary = hsl(0.0, 0.0, model.amount as f32 / 100.0);

    draw.background().color(secondary);
    draw.text("FOLTIK")
        .font(model.font.clone())
        .font_size(96)
        .wh(app.main_window().rect().wh())
        .color(primary);

    draw.to_frame(app, &frame).unwrap();
}
