use gfx::animation::Animator;
use gfx::pass::{FilterPass, RingPass, TextPass, TextPassBuilder, ImagePass};
use gfx::scene::Scene;
use lib::prelude::*;
use lib::resource;
use lib::twitch::TwitchBuffer;

mod pipeline;

mod animations;
use animations::animations;

mod beat;
use beat::Beat;

mod decay;
use decay::Decay;

mod mixxx;
use mixxx::Mixxx;

mod stage;
use stage::{Stage};

fn main() {
    lib::app::run(model, input, update, view);
}

pub struct Model {
    audio: Audio,
    osc: Osc,

    twitch: TwitchBuffer,
    mixxx: Mixxx,

    t: f32,
    tc: f32,
    t_mul: f32,

    beat: Beat,
    beat_v: bool,
    decay: Decay,

    stage: Option<Stage>,

    // text: TextPass,
    fx: pipeline::Fx,
    ring: RingPass,
}

async fn model(app: &App) -> Model {
    let device = &app.device;

    // let images = ImagePass::new(device)
    //     .with(app, "cockpit.png", "cockpit", v2(200.0, 200.0), v2(1.0, 1.0));
    // let text = TextPassBuilder::new()
    //     .with("default", "go-b.ttf")
    //     .build(device);
    let fx = pipeline::Fx::new(device);
    let ring = RingPass::new(device, 4);

    Model {
        tc: 0.0,
        t: 0.0,
        t_mul: 1.0,

        audio: Audio::default(),
        osc: Osc::new("0.0.0.0:7777"),

        twitch: TwitchBuffer::new(50),
        mixxx: Mixxx::default(),

        beat: Beat::default(),
        beat_v: false,
        decay: Decay::default(),

        stage: Some(Stage::new(app)),

        fx,
        // text,
        ring,
    }
}

async fn input(_app: &App, m: &mut Model, state: KeyState, key: Key) {
    if state != KeyState::Pressed {
        return;
    }

    match key {
        Key::Space => m.beat.beat(),
        Key::Key1 => m.mixxx.decks[0].toggle(),
        Key::Key2 => m.mixxx.decks[1].toggle(),
        Key::Return => m.stage.as_mut().unwrap().next(),
        _ => {}
    }
}

fn midi(m: &mut Model, bank: MidiBank, msg: Midi) {
    m.beat.midi(msg);
    m.fx.midi(msg);

    match msg {
        Midi::Knob(6, f) => m.t_mul = f,
        _ => {}
    }

    m.stage = Some(m.stage.take().unwrap().midi(m, bank, msg));
}

fn osc(m: &mut Model, msg: OscMessage) {
    m.mixxx.osc(msg);
}

async fn update(app: &App, m: &mut Model, dt: f32) {
    m.twitch.update();
    m.audio.update();
    m.audio.midi().into_iter().for_each(|(b, msg)| midi(m, b, msg));
    m.osc.poll().into_iter().for_each(|msg| osc(m, msg));

    let dt_mod = dt * m.t_mul * (600.0 * m.audio.rms());
    // let dt_mod = dt * 0.5;
    m.tc += dt;
    m.t += dt_mod;

    m.decay.update(dt);
    m.fx.update(m.tc, m.t);

    let beat = m.beat.update(dt, &mut m.audio, &m.mixxx);
    if beat {
        m.decay.beat_set();
        m.beat_v = true;
    }

    m.stage = Some(m.stage.take().unwrap().update(app, m, dt));

    m.ring.update();

    m.beat_v = false;
}

async fn view(app: &mut App, m: &mut Model, view: &wgpu::SwapChainTextureView) {
    let frame = &mut Frame::new(app);

    m.fx.upload(frame);

    m.stage = Some(m.stage.take().unwrap().view(frame, m.fx.view()));

    m.fx.encode(frame, m.ring.view());
    m.ring.encode(frame, view);

    frame.submit();
}
