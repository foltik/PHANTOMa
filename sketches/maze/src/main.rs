use nannou::prelude::*;
use nannou::text::{font, Font};
use nannou::geom::Range;
use num::integer::Integer;
use std::time::Instant;
use itertools::Itertools;

mod scenes;
mod shapes;

use scenes::{Cube, Church, Maze, Pillars, Temple};

use lib::{
    self,
    audio::{self, Audio},
    gfx::{Composite, Drawer, Effect, Present},
    midi::{Midi, MidiMessage, MidiBank},
    osc::{Osc, OscMessage},
    time::{BeatClock, BeatDetect, DecayEnv},
    twitch::TwitchBuffer,
};

#[tokio::main]
async fn main() {
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
    edge: f32,
    mega: f32,
}

#[derive(Default)]
struct Params {
    index: usize,
    t_mul: f32,
    beatstop: bool,
    manual: bool,
    net: bool,
    red: f32,
    edge: f32,
    vglitch: f32,
    vliq: f32,
    tnames: f32,
    fnames: f32,
}

struct AppModel {
    audio: Box<dyn Audio>,
    midi: Midi,
    osc: Osc,
    twitch: TwitchBuffer,

    beat_detect: BeatDetect,
    beat_clock: BeatClock,

    t: f32,
    t_pause: f32,

    param: Params,
    decay: DecayEnv,
    effect_state: EffectState,

    font: Font,
    mfont: Font,
    rfont: Font,
    gifont: Font,

    chatbox: wgpu::Texture,
    code0: String,
    code1: String,
    code2: String,
    code3: String,

    cube: Cube,
    maze: Maze,
    pillars: Pillars,
    temple: Temple,
    church: Church,

    drawer: Drawer,
    composite0: Composite,
    edge: Effect<EffectState>,
    edge2: Effect<EffectState>,
    shake: Effect<EffectState>,
    glitch: Effect<EffectState>,
    vhs: Effect<EffectState>,
    pause: Effect<EffectState>,
    fade: Effect<EffectState>,
    hud: Drawer,
    composite1: Composite,
    composite2: Composite,
    present: Present,
}

fn model(app: &App) -> AppModel {
    let window_id = app
        .new_window()
        .key_pressed(key_pressed)
        .size(1920, 1080)
        .title("PHANTOMa")
        .view(view)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let device = window.swap_chain_device();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    let cube = Cube::new(device, &window, &mut encoder);
    let maze = Maze::new(device, &window, &mut encoder);
    let pillars = Pillars::new(device, &window, &mut encoder);
    let temple = Temple::new(device, &window, &mut encoder);
    let church = Church::new(device, &window, &mut encoder);

    let edge = Effect::new(device, "edge.frag.spv");
    let edge2 = Effect::new(device, "edge.frag.spv");
    let shake = Effect::new(device, "shake.frag.spv");
    let glitch = Effect::new(device, "glitch.frag.spv");
    let vhs = Effect::new(device, "vhs.frag.spv");
    let fade = Effect::new(device, "fade.frag.spv");
    let pause = Effect::new(device, "pause.frag.spv");
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
        edge: 0.0,
        mega: 0.0,
    };

    let param = Params::default();

    let decay = DecayEnv::new()
        .with("glitch", 100.0)
        .with("red", 250.0)
        .with("flash", 200.0)
        .with("light", 1000.0)
        .with("edge", 1000.0);

    AppModel {
        audio: Box::new(audio::init()),
        midi: Midi::init(),
        osc: Osc::init(34254),
        twitch: TwitchBuffer::init(12),

        beat_detect: BeatDetect::new(40.0, 120.0, 0.005, 400.0),
        beat_clock: BeatClock::new(1.0),

        t: 0.0,
        t_pause: 0.0,

        param,
        decay,
        effect_state: state,

        font: font::from_file(lib::resource("magi.ttf")).unwrap(),
        mfont: font::from_file(lib::resource("pelagiad.ttf")).unwrap(),
        rfont: font::from_file(lib::resource("random.ttf")).unwrap(),
        gifont: font::from_file(lib::resource("go-bi.ttf")).unwrap(),

        chatbox: wgpu::Texture::from_path(app, lib::resource("dialogue.png")).unwrap(),

        code0: lib::read_resource("code0.txt"),
        code1: lib::read_resource("code1.txt"),
        code2: lib::read_resource("code2.txt"),
        code3: lib::read_resource("code3.txt"),

        cube,
        maze,
        pillars,
        temple,
        church,

        drawer: Drawer::new(device, 4),
        composite0: Composite::new(device),
        edge,
        edge2,
        shake,
        glitch,
        vhs,
        fade,
        pause,
        hud: Drawer::new(device, 4),
        composite1: Composite::new(device),
        composite2: Composite::new(device),
        present,
    }
}

fn key_pressed(_app: &App, model: &mut AppModel, key: Key) {
    match key {
        Key::Left => model.param.manual = true,
        Key::Right => model.param.manual = true,
        _ => {}
    };
}

fn update(app: &App, model: &mut AppModel, update: Update) {
    let ms = update.since_last.as_nanos() as f32 / 1_000_000.0;
    let start = Instant::now();

    log::trace!(
        "Draw in {:?} / {}ups",
        update.since_last,
        1.0 / (ms / 1000.0)
    );

    model.audio.update();
    model.twitch.update();

    let mut edge = model.decay.v("edge") + model.param.edge;

    for (bank, message) in model.midi.poll() {
        match bank {
            MidiBank::B0 => match message {
                MidiMessage::Slider(8, f) => model.pillars.door = f,
                _ => {}
            }
            MidiBank::B1 => match message {
                _ => {}
            }
            MidiBank::B2 => match message {
                _ => {}
            }
            MidiBank::B3 => match message {
                MidiMessage::MainButton(7, true) => {
                    let start = model.param.index;
                    let mut i = start;
                    while i == start {
                        i = random_range(5, 11);
                    }
                    model.param.index = i;
                },
                MidiMessage::MainButton(8, b) => model.church.beat = b,
                MidiMessage::Slider(8, f) => model.church.offset = f,
                _ => {
                }
            }
        }

        match message {
            // shader fx
            MidiMessage::Slider(0, f) => model.param.red = f,
            MidiMessage::Slider(1, f) => model.effect_state.glitch = f,
            MidiMessage::Slider(2, f) => model.effect_state.vhs = f,
            MidiMessage::Slider(4, f) => model.param.vliq = f,
            MidiMessage::Slider(5, f) => model.param.edge = f,
            MidiMessage::Slider(3, f) => model.param.vglitch = f,
            MidiMessage::Slider(6, f) => {
                model.effect_state.mega = f;
                model.cube.scene.fx.vals.y = f;
                model.pillars.scene.fx.vals.y = f;
                model.maze.scene.fx.vals.y = f;
                model.temple.scene.fx.vals.y = f;
                model.church.scene.fx.vals.y = f;
            },
            MidiMessage::Slider(7, f) => model.effect_state.pause = f,
            MidiMessage::Knob(0, f) => model.effect_state.black = f,
            MidiMessage::Knob(1, f) => model.param.tnames = f,
            MidiMessage::Knob(2, f) => model.param.fnames = f,

            MidiMessage::MainButton(5, true) => model.decay.set("edge"),

            // Swap scene
            MidiMessage::TopButton(0, true) => {
                model.param.index = std::cmp::max(model.param.index, 1) - 1;
            }
            MidiMessage::TopButton(1, true) => {
                model.effect_state.glitch = 0.0;
                model.param.index = (model.param.index + 1).mod_floor(&12)
            }

            // Time
            //MidiMessage::MainButton(8, true) => model.decay.set("flash"),

            // Beat Control
            MidiMessage::BankButton(0, b) => model.param.beatstop = b,
            MidiMessage::BankButton(1, b) => model.param.beatstop = b,

            MidiMessage::CtrlButton(0, true) => model.beat_clock.sync(),
            MidiMessage::CtrlButton(1, true) => model.beat_clock.mul = 2.0,
            MidiMessage::CtrlButton(2, true) => model.beat_clock.mul = 1.0,
            MidiMessage::CtrlButton(3, true) => model.beat_clock.mul = 0.5,
            MidiMessage::CtrlButton(4, true) => model.beat_clock.mul = 0.25,
            MidiMessage::CtrlButton(5, t) => model.param.net = t,
            MidiMessage::Knob(6, f) => model.param.t_mul = f,
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

    model.effect_state.edge = edge;

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
        || model.param.manual;

    if beat {
        model.decay.set("red");
        model.decay.set("light");
        model.decay.set("glitch");
    }

    model.effect_state.red = model.param.red * 2.0 * model.decay.v("red");
    model.effect_state.flash = model.decay.v("flash");

    model.cube.scene.fx.vals.x = model.t_pause;
    model.cube.scene.fx.vals.y = model.param.vglitch;
    model.cube.scene.fx.vals.z = model.param.vliq;
    model.pillars.scene.fx.vals.x = model.t_pause;
    model.pillars.scene.fx.vals.y = model.param.vglitch;
    model.pillars.scene.fx.vals.z = model.param.vliq;
    model.maze.scene.fx.vals.x = model.t_pause;
    model.maze.scene.fx.vals.y = model.param.vglitch;
    model.maze.scene.fx.vals.z = model.param.vliq;
    model.temple.scene.fx.vals.x = model.t_pause;
    model.temple.scene.fx.vals.y = model.param.vglitch;
    model.temple.scene.fx.vals.z = model.param.vliq;
    model.church.scene.fx.vals.x = model.t_pause;
    model.church.scene.fx.vals.y = model.param.vglitch;
    model.church.scene.fx.vals.z = model.param.vliq;

    if model.param.index == 0 {
        model.effect_state.glitch = model.audio.rms() * 80.0;
        model.cube.update(model.t_pause / 100.0);
    }

    if model.param.index == 1 || model.param.index == 2 {
        model.pillars.brightness = model.decay.v("light");
        model
            .pillars
            .update(model.param.index - 1, app.time, model.t_pause / 100.0);
    }

    if model.param.index == 3 {
        model.maze.update(model.t_pause / 300.0);
    }

    if model.param.index == 4 {
        model.temple.brightness = model.decay.v("light");
        model.temple.red = model.param.red;
        model.temple.update(model.t_pause / 100.0);
    }

    if model.param.index >= 5 {
        model.church.brightness = model.decay.v("light");
        if model.church.beat && beat {
            let start = model.param.index;
            let mut i = start;
            while i == start {
                i = random_range(5, 11);
            }
            model.param.index = i;
        }

        model
            .church
            .update(model.param.index - 5, app.time, model.t_pause / 50.0);
    }

    model.param.manual = false;

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

        let window = app.main_window();
        let device = window.swap_chain_device();
        let mut encoder = frame.command_encoder();

        if model.param.index == 0 {
            model.cube.draw(device, &mut encoder, &model.edge.view);
        } if model.param.index == 1 || model.param.index == 2 {
            model.pillars.draw(device, &mut encoder, &model.edge.view);
        } else if model.param.index == 3 {
            model
                .maze
                .draw(device, &mut encoder, &model.composite0.view1);

            shapes::demon1(
                &draw,
                model.font.clone(),
                &pt2(0.0, 0.0),
                200.0 + 100.0 * model.decay.v("red"),
                model.t_pause,
            );

            model
                .drawer
                .encode(device, &mut encoder, &model.composite0.view2, &draw);

            model.composite0.encode(&mut encoder, &model.edge.view);
        } else if model.param.index == 4 {
            model.temple.draw(device, &mut encoder, &model.edge.view);
        } else if model.param.index >= 5 {
            model.church.draw(device, &mut encoder, &model.edge.view);
        }

        model.edge.update(device, &mut encoder, &model.effect_state);
        model.edge.encode(&mut encoder, &model.shake.view);

        model
            .shake
            .update(device, &mut encoder, &model.effect_state);
        model.shake.encode(&mut encoder, &model.glitch.view);

        model
            .glitch
            .update(device, &mut encoder, &model.effect_state);
        model.glitch.encode(&mut encoder, &model.vhs.view);

        model.vhs.update(device, &mut encoder, &model.effect_state);
        model.vhs.encode(&mut encoder, &model.pause.view);

        model
            .pause
            .update(device, &mut encoder, &model.effect_state);
        model.pause.encode(&mut encoder, &model.fade.view);

        model.fade.update(device, &mut encoder, &model.effect_state);
        model.fade.encode(&mut encoder, &model.composite1.view1);

        draw.reset();
        draw.background().color(BLACK);

        let messages = model.twitch.latest().map(|m| format!("[{}]:  {}", m.user, m.body)).join("\n");
        draw.text(&messages)
            .color(Rgba::new(0.698, 0.627, 0.471, 1.0))
            .align_text_bottom()
            .left_justify()
            .line_spacing(7.0)
            .font(model.mfont.clone())
            .x_y(-750.0, -348.0)
            .height(210.0)
            .width(360.0);

        draw.rect()
            .color(BLACK)
            .x_y(-750.0, 54.0)
            .height(610.0)
            .width(360.0);

        draw.scale(0.7).texture(&model.chatbox).x_y(-980.0, -500.0);

        draw.rect().color(Rgba::new(0.0, 0.0, 0.0, 0.85)).width(1920.0).height(1080.0);

        if model.param.index == 0 {
            shapes::orbit(&draw, &Point2::new(0.0, 0.0), 400.0, model.t_pause, 1.0 - model.effect_state.black);
        }

        shapes::rnames(&draw, model.rfont.clone(), model.param.tnames, app.time);
        shapes::fnames(&draw, model.rfont.clone(), model.param.fnames, app.time);

        model
            .hud
            .encode(device, &mut encoder, &model.composite1.view2, &draw);

        model.composite1.encode(&mut encoder, &model.composite2.view1);

        draw.reset();
        draw.background().color(BLACK);

        if model.param.index == 0 {
            let draw = draw.scale(Range::new(1.0, 1.01).lerp(model.decay.v("light")));
            let color = Rgba::new(1.0, 1.0, 1.0, 1.0 - model.effect_state.black);
            draw.text("INiT PR0JECT PHANTOMa").font(model.gifont.clone()).color(color).y(490.0).font_size(54).width(1000.0);
            draw.text("THOMAS LEGACY").font(model.gifont.clone()).color(color).x_y(-680.0, 340.0).font_size(40).width(1000.0);
            draw.text("FOLTIK").font(model.gifont.clone()).color(color).x_y(780.0, 190.0).font_size(40).width(1000.0);
        }

        model
            .hud
            .encode(device, &mut encoder, &model.edge2.view, &draw);

        let mut tstate = model.effect_state.clone();
        tstate.edge = model.decay.v("light");

        model.edge2.update(device, &mut encoder, &tstate);
        model.edge2.encode(&mut encoder, &model.composite2.view2);
        model.composite2.encode(&mut encoder, model.present.view());

        model.present.encode(&mut encoder, &frame);
    }

    let elapsed = start.elapsed();
    log::trace!(
        "Frame encoded in {:?} / {}fps",
        elapsed,
        1.0 / (elapsed.as_micros() as f32 / 1_000_000.0)
    );
}
