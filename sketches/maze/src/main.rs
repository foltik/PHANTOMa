use nannou::prelude::*;
use nannou::text::{font, Font};
use num::integer::Integer;
use std::time::Instant;

mod scenes;
mod shapes;

use scenes::{Church, Maze, Pillars, Temple};

use lib::{
    self,
    audio::{self, Audio},
    gfx::{Composite, Drawer, Effect, Present},
    midi::{Midi, MidiMessage},
    osc::{Osc, OscMessage},
    time::{BeatClock, BeatDetect, DecayEnv},
};

fn main() {
    lib::init_logging(2);
    nannou::app(model).update(update).view(view).run();
}

// Uniform to control the Glitch shader parameters
#[derive(Copy, Clone, Debug)]
struct EffectState {
    t: f32,
    tc: f32,
    pause: f32,
    glitch: f32,
    glitch_mo: f32,
    vhs: f32,
    red: f32,
    flash: f32,
    shake: f32,
    black: f32,
}

#[derive(Default)]
struct Params {
    index: u32,
    t_mul: f32,
    beatstop: bool,
    net: bool,
    red: f32,
    zoom: f32,
}

struct AppModel {
    audio: Box<dyn Audio>,
    midi: Midi,
    osc: Osc,

    beat_detect: BeatDetect,
    beat_clock: BeatClock,

    t: f32,
    t_pause: f32,

    param: Params,
    decay: DecayEnv,
    effect_state: EffectState,

    font: Font,

    maze: Maze,
    pillars: Pillars,
    temple: Temple,
    church: Church,

    drawer: Drawer,

    composite: Composite,
    glitch: Effect<EffectState>,
    present: Present,
}

fn model(app: &App) -> AppModel {
    let window_id = app
        .new_window()
        .size(1920, 1080)
        .title("PHANTOMa")
        .view(view)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let device = window.swap_chain_device();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    let maze = Maze::new(device, &window, &mut encoder);
    let pillars = Pillars::new(device, &window, &mut encoder);
    let temple = Temple::new(device, &window, &mut encoder);
    let church = Church::new(device, &window, &mut encoder);

    let glitch = Effect::new(device, "glitch.frag.spv");
    let present = Present::new(device, window.msaa_samples());

    window
        .swap_chain_queue()
        .lock()
        .unwrap()
        .submit(&[encoder.finish()]);

    let state = EffectState {
        t: 0.0,
        tc: 0.0,
        pause: 0.0,
        glitch: 0.0,
        glitch_mo: 0.0,
        vhs: 0.0,
        red: 0.0,
        flash: 0.0,
        shake: 0.0,
        black: 0.0,
    };

    let param = Params::default();

    let decay = DecayEnv::new()
        .with("glitch", 100.0)
        .with("red", 250.0)
        .with("flash", 200.0)
        .with("light", 1000.0);

    AppModel {
        audio: Box::new(audio::init()),
        midi: Midi::init(),
        osc: Osc::init(34254),

        beat_detect: BeatDetect::new(40.0, 120.0, 0.005, 400.0),
        beat_clock: BeatClock::new(1.0),

        t: 0.0,
        t_pause: 0.0,

        param,
        decay,
        effect_state: state,

        font: font::from_file("../../resources/fonts/magi.ttf").unwrap(),

        maze,
        pillars,
        temple,
        church,

        drawer: Drawer::new(device, 4),

        composite: Composite::new(device),
        glitch,
        present,
    }
}

fn update(app: &App, model: &mut AppModel, update: Update) {
    let ms = update.since_last.as_nanos() as f32 / 1_000_000.0;
    let start = Instant::now();

    log::debug!(
        "Draw in {:?} / {}ups",
        update.since_last,
        1.0 / (ms / 1000.0)
    );

    model.audio.update();

    let mut beat_manual = false;

    let mut dir = Vector3::new(0.0, 0.0, 0.0);

    for (_, message) in model.midi.poll() {
        match message {
            // shader fx
            MidiMessage::Slider(0, f) => model.param.red = f,
            MidiMessage::Slider(1, f) => model.effect_state.glitch = f,
            MidiMessage::Slider(2, f) => model.effect_state.vhs = f,
            MidiMessage::Slider(3, f) => model.effect_state.pause = f,
            MidiMessage::Slider(4, f) => model.effect_state.black = f,

            // Swap scene
            MidiMessage::TopButton(0, true) => {
                model.param.index = (model.param.index - 1).mod_floor(&4)
            }
            MidiMessage::TopButton(1, true) => {
                model.param.index = (model.param.index + 1).mod_floor(&4)
            }

            // Time
            MidiMessage::Knob(6, f) => model.param.t_mul = f,

            MidiMessage::MainButton(0, true) => dir.x = -1.0,
            MidiMessage::MainButton(1, true) => dir.z = -1.0,
            MidiMessage::MainButton(2, true) => dir.z = 1.0,
            MidiMessage::MainButton(3, true) => dir.x = 1.0,
            MidiMessage::MainButton(4, true) => dir.y = -1.0,
            MidiMessage::MainButton(5, true) => dir.y = 1.0,

            // Effect Buttons
            MidiMessage::MainButton(7, true) => beat_manual = true,
            MidiMessage::MainButton(8, true) => model.decay.set("flash"),

            MidiMessage::Slider(5, f) => {
                model.church.scene.models[0].objects[39]
                    .material
                    .desc
                    .emissive
                    .col = nannou::math::cgmath::Vector3::new(0.786, 0.098, 0.048)
                    + nannou::math::cgmath::Vector3::from_value(0.2) * f
            }
            MidiMessage::Slider(6, f) => model.param.zoom = f * 0.3,
            MidiMessage::Slider(7, f) => model.param.zoom = f,
            MidiMessage::Slider(8, f) => model.pillars.door = f,

            // Beat Control
            MidiMessage::CtrlButton(0, true) => model.beat_clock.sync(),
            MidiMessage::CtrlButton(1, true) => model.beat_clock.mul *= 2.0,
            MidiMessage::CtrlButton(2, true) => model.beat_clock.mul /= 2.0,
            MidiMessage::CtrlButton(3, t) => model.param.beatstop = t,
            MidiMessage::CtrlButton(4, t) => model.param.beatstop = t,
            MidiMessage::CtrlButton(5, t) => model.param.net = t,
            MidiMessage::Knob(7, f) => model.beat_detect.bpm_max = 200.0 + f * 300.0,
            MidiMessage::Knob(8, f) => model.beat_detect.thres = 0.1 * f,
            _ => {}
        }
    }

    for msg in model.osc.poll() {
        match msg {
            OscMessage::Bpm(bpm) => model.beat_clock.bpm = bpm,
            _ => {}
        }
    }

    model.decay.update(ms);

    // Update times
    let t_mod = (model.param.t_mul * 75.0) * ms * model.audio.rms();
    model.t += t_mod;
    model.t_pause += t_mod * (1.0 - model.effect_state.pause);
    model.effect_state.t = model.t;
    model.effect_state.tc = app.time;

    // Check if we're beating
    let audio_beat = model.beat_detect.update(ms, &*model.audio);
    let clock_beat = model.beat_clock.update(ms);
    let beat = (!model.param.beatstop
        && model.effect_state.pause == 0.0
        && if model.param.net {
            clock_beat
        } else {
            audio_beat
        })
        || beat_manual;

    if beat {
        model.decay.set("red");
        model.decay.set("light");
        model.decay.set("glitch");
    }

    if model.param.index == 1 {
        model.effect_state.red = model.param.red * 2.0 * model.decay.v("red");
    }

    model.effect_state.flash = model.decay.v("flash");

    model.pillars.brightness = model.decay.v("light");
    model.pillars.zoom = model.param.zoom;
    model.pillars.update();

    model.maze.update(model.t_pause / 300.0);

    model.temple.update(model.t_pause / 100.0);

    model.church.scene.camera.desc.eye += dir.into();
    model.church.update(app.time, model.t_pause / 100.0);

    let elapsed = start.elapsed();
    log::trace!(
        "Update in {:?} / {}ups",
        elapsed,
        1.0 / (elapsed.as_micros() as f32 / 1_000_000.0)
    );
}

fn view(app: &App, model: &AppModel, frame: Frame) {
    let start = Instant::now();

    {
        let draw = app.draw();
        draw.background().color(BLACK);
        if model.param.index == 1 {
            shapes::demon1(
                &draw,
                model.font.clone(),
                &pt2(0.0, 0.0),
                200.0 + 100.0 * model.decay.v("red"),
                model.t_pause,
            );
        }

        let window = app.main_window();
        let device = window.swap_chain_device();
        let mut encoder = frame.command_encoder();

        if model.param.index == 0 {
            model.pillars.draw(device, &mut encoder, &model.glitch.view);
        } else if model.param.index == 1 {
            model
                .maze
                .draw(device, &mut encoder, &model.composite.view1);

            model
                .drawer
                .encode(device, &mut encoder, &model.composite.view2, &draw);

            model.composite.encode(&mut encoder, model.glitch.view());
        } else if model.param.index == 2 {
            model.temple.draw(device, &mut encoder, &model.glitch.view);
        } else if model.param.index == 3 {
            model.church.draw(device, &mut encoder, &model.glitch.view);
        }

        model
            .glitch
            .update(device, &mut encoder, &model.effect_state);
        model.glitch.encode(&mut encoder, model.present.view());

        model.present.encode(&mut encoder, &frame);
    }

    let elapsed = start.elapsed();
    log::debug!(
        "Frame encoded in {:?} / {}fps",
        elapsed,
        1.0 / (elapsed.as_micros() as f32 / 1_000_000.0)
    );
}
