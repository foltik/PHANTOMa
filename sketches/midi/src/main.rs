use nannou::prelude::*;
use lib::{Decay, midi::{Midi, MidiMessage}};

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
    midi: Midi,
    fr: f32,
    weight: f32,
    rotate: f32,
    bounce: Decay,
    sides: i8,
}

fn model(_app: &App) -> Model {
    Model {
        midi: Midi::init(),
        fr: 0.0,
        weight: 1.0,
        rotate: 0.0,
        bounce: Decay::new(),
        sides: 0,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    for (bank, message) in model.midi.poll() {
        println!("[{:?}]: {:?}", bank, message);

        match message {
            MidiMessage::Slider(0, f) => model.fr = f,
            MidiMessage::Slider(1, f) => model.weight = f,
            MidiMessage::Knob(0, f) => model.rotate = f * TAU / 4.0,
            MidiMessage::MainButton(0, true) => model.bounce.set_max(),
            MidiMessage::Encoder(i) => {
                let v = model.sides + i;
                model.sides = ((v % 12) + 12) % 12; // rust only does remainder modulo...
            }
            _ => {}
        }
    }

    model.bounce.update(update.since_last.as_millis() as f32 / 250.0);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw().rotate(model.rotate);

    draw.background().color(BLACK);

    draw.ellipse()
        .no_fill()
        .stroke(WHITE)
        .stroke(hsv(0.76, model.bounce.v(), 1.0))
        .stroke_weight(1.0 + (model.weight + model.bounce.v()) * 15.0)
        .radius(200.0 * model.fr)
        .resolution((model.sides + 3) as usize);

    draw.to_frame(app, &frame).unwrap();
}
