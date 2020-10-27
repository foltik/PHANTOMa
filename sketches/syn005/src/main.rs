use gfx::animation::Animator;
use gfx::pass::{FilterPass, RingPass, TextPass, TextPassBuilder};
use gfx::scene::Scene;
use lib::prelude::*;
use lib::resource;

mod pipeline;

mod animations;
use animations::animations;

mod beat;
use beat::Beat;

mod decay;
use decay::Decay;

fn main() {
    lib::app::run(model, input, update, view);
}

struct Model {
    t: f32,
    tc: f32,
    t_mul: f32,

    audio: Audio,
    beat: Beat,
    decay: Decay,

    scene: Scene,
    animator: Animator,

    phong: pipeline::Phong,
    animated: pipeline::Animated,

    text: TextPass,
    composite: FilterPass,
    fx: pipeline::Fx,
    ring: RingPass,
}

async fn model(app: &App) -> Model {
    let device = &app.device;

    let scene = resource::read_scene(device, "test.glb");
    let animator = Animator::new(&scene);

    let phong = pipeline::Phong::new(app, &scene, |name| name != "Plane" && name != "IcoLight");

    let animated = pipeline::Animated::new(app, &scene, &animations());

    let text = TextPassBuilder::new()
        .with("default", "pelagiad.ttf")
        .build(device);
    let composite = FilterPass::new_composite::<()>(device, "composite", 3, None, None);
    let fx = pipeline::Fx::new(device);
    let ring = RingPass::new(device, 4);

    Model {
        tc: 0.0,
        t: 0.0,
        t_mul: 1.0,

        audio: Audio::default(),
        beat: Beat::default(),
        decay: Decay::default(),

        scene,
        animator,

        phong,
        animated,

        fx,
        text,
        composite,
        ring,
    }
}

async fn input(_app: &App, m: &mut Model, state: KeyState, key: Key) {
    if state != KeyState::Pressed {
        return;
    }

    match key {
        Key::Space => m.decay.beat_set(),
        Key::A => m.animator.play(m.t, false, "IcosphereAction"),
        _ => {}
    }
}

fn midi(m: &mut Model, _bank: MidiBank, msg: Midi) {
    m.beat.midi(msg);
    m.fx.midi(msg);

    match msg {
        Midi::Knob(6, f) => m.t_mul = f,
        _ => {}
    }
}

async fn update(_app: &App, m: &mut Model, dt: f32) {
    m.audio.update();
    m.audio.midi().iter().for_each(|(b, msg)| midi(m, *b, *msg));

    let dt_mod = dt * (m.t_mul * 200.0) * m.audio.rms();
    m.tc += dt;
    m.t += dt_mod;

    m.decay.update(dt);
    m.fx.update(m.tc, m.t);

    let beat = m.beat.update(dt, &mut m.audio);
    if beat {
        m.decay.beat_set();
    }

    m.text.draw(|d| {
        d.at(v2(200.0, 200.0))
            .text("Test", |t| t.scale(62.0).color(v4(1.0, 0.0, 0.0, 1.0)))
    });

    let scale = 1.0 + (1.0 * m.decay.beat());
    m.scene.node_mut("Icosphere").transform.scale = Vector3::new(scale, scale, scale);
    m.animator.update(m.t, &mut m.scene);

    m.animated.update(m.tc);
    m.ring.update();
}

async fn view(app: &mut App, m: &mut Model, view: &wgpu::SwapChainTextureView) {
    let frame = &mut Frame::new(app);

    m.scene.update(frame);
    m.animated.upload(frame);
    m.fx.upload(frame);

    m.phong.encode(frame, &m.scene, m.composite.view(0));
    m.animated.encode(frame, &m.scene, m.composite.view(1));
    m.text.encode(frame, m.composite.view(2));

    m.composite.encode(frame, m.fx.view());
    m.fx.encode(frame, m.ring.view());
    m.ring.encode(frame, view);

    frame.submit();
}
