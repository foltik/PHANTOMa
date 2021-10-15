use lib::prelude::*;

use lib::resource;
use gfx::scene::Scene;
use gfx::animation::Animator;
use gfx::pass::{FilterPass, RingPass, TextPass, TextPassBuilder, ImagePass, SynthPass};
use lib::twitch::TwitchBuffer;

use crate::Model;
use crate::pipeline;

#[derive(Clone, Copy)]
#[repr(C)]
struct Ui {
    f0: f32,
    f1: f32,
    f2: f32,
    beat: f32,
}

pub struct Canals {
    init: bool,

    scene: Scene,
    animator: Animator,
    ui: UniformStorage<Ui>,

    phong: pipeline::Phong,
    ui_synth: SynthPass,
    images: ImagePass,
    text: TextPass,
    composite: FilterPass,
    depth: wgpu::TextureView,
}

impl Canals {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let scene = resource::read_scene(device, "canals.glb");

        let animator = Animator::new(&scene);

        let ui = UniformStorage::new(device, "ui", Ui {
            f0: 90.0,
            f1: 230.0,
            f2: 150.0,
            beat: 1.0,
        });

        let phong = pipeline::Phong::new(app, &scene, |name| true, |mat| true);
        let ui_synth = SynthPass::new(device, "ui", "cockpit.frag.spv", Some(ui.as_ref()));
        let images = ImagePass::new(device)
            .with(app, "cockpit.png", "cockpit", v2(0.0, 0.0), v2(1.0, 1.0));
        let text = TextPassBuilder::new()
            .with("default", "go-b.ttf")
            .build(device);
        let composite = FilterPass::new_composite::<()>(device, "composite", 3, None, None);

        let depth = wgpu::util::TextureBuilder::new_depth("depth")
            .build(&app.device)
            .view()
            .build();

        Self {
            init: false,

            scene,
            animator,
            ui,

            phong,
            ui_synth,
            images,
            text,
            composite,
            depth,
        }
    }

    pub fn input(&mut self, app: &App, m: &mut Model, state: KeyState, key: Key) {

    }

    pub fn midi(&mut self, m: &mut Model, _bank: MidiBank, msg: Midi) {

    }

    fn init(&mut self, app: &App, m: &mut Model, dt: f32) {
        self.animator.play(m.t, true, "Action.003");
        self.animator.play(m.t, true, "Action.005");
        self.animator.play(m.t, true, "Action.004");
    }

    pub fn update(&mut self, app: &App, m: &mut Model, dt: f32) {
        if !self.init {
            self.init(app, m, dt);
            self.init = true;
        }

        let chat = m
            .twitch
            .block(37, 20)
            .join("\n");
        self.text.draw(|d| {
            d.at(v2(577.0, 682.0))
                .text(&chat, |t| t.scale(12.0).color(v4(0.0, 1.0, 0.0, 0.8)))
        });

        if m.mixxx.decks[0].active {
            self.text.draw(|d| {
                d.at(v2(10.0, 1080.0 - 80.0))
                    .text(&m.mixxx.decks[0].song, |t| t.scale(12.0).color(v4(0.0, 1.0, 0.0, 1.0)))
            })
        }
        if m.mixxx.decks[1].active {
            self.text.draw(|d| {
                d.at(v2(10.0, 1080.0 - 100.0))
                    .text(&m.mixxx.decks[1].song, |t| t.scale(12.0).color(v4(0.0, 1.0, 0.0, 1.0)))
            })
        }

        let clamp = |f| f32::max(0.1, f32::min(1.0, f));

        self.ui.beat = m.decay.beat();
        self.ui.f0 = clamp(m.audio.rms_range(40.0, 120.0) * 100.0);
        self.ui.f1 = clamp(m.audio.rms_range(200.0, 2000.0) * 100.0);
        self.ui.f2 = clamp(m.audio.rms_range(3000.0, 5000.0) * 100.0);

        self.animator.update(m.t, &mut self.scene);
    }

    pub fn view(&mut self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        self.ui.upload(frame);

        self.scene.update(frame);
        self.phong.encode(frame, &self.scene, &self.depth, self.composite.view(0));
        self.images.encode(frame, self.composite.view(1));
        self.text.encode(frame, self.composite.view(1));
        self.ui_synth.encode(frame, self.composite.view(2));

        self.composite.encode(frame, target);
    }
}