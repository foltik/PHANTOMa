use lib::prelude::*;

use lib::resource;
use gfx::scene::Scene;
use gfx::animation::Animator;
use gfx::pass::{FilterPass, RingPass, TextPass, TextPassBuilder, ImagePass, SynthPass};
use lib::twitch::TwitchBuffer;

use crate::Model;
use crate::pipeline;

pub struct Stasis {
    init: bool,

    scene: Scene,
    animator: Animator,
    uniform: UniformStorage<f32>,

    phong: pipeline::Phong,
    psych: SynthPass,
    text: TextPass,
    composite: FilterPass,
    depth: wgpu::TextureView,
}

impl Stasis {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let scene = resource::read_scene(device, "stasis.glb");

        let animator = Animator::new(&scene);

        let uniform = UniformStorage::new(device, "psych", 0.0);

        let phong = pipeline::Phong::new(app, &scene, |name| true, |mat| true);
        let psych = SynthPass::new(device, "psych", "psych.frag.spv", Some(uniform.as_ref()));
        let text = TextPassBuilder::new()
            .with("default", "go-b.ttf")
            .build(device);
        let composite = FilterPass::new_composite::<()>(device, "composite", 2, None, None);

        let depth = wgpu::util::TextureBuilder::new_depth("depth")
            .build(&app.device)
            .view()
            .build();

        Self {
            init: false,

            scene,
            animator,
            uniform,

            phong,
            psych,
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
        self.animator.play(m.t, true, "CameraAction");

        self.animator.play(m.t, true, "PointAction.001");
        self.animator.play(m.t, true, "PointAction.002");

        for i in 0..=12 {
            self.animator.play(m.t, true, &format!("CubeAction.0{:02}", i))
        }
    }

    pub fn update(&mut self, app: &App, m: &mut Model, dt: f32) {
        if !self.init {
            self.init(app, m, dt);
            self.init = true;
        }

        *self.uniform = m.t;

        let chat = m
            .twitch
            .block(120, 20)
            .join("\n");
        self.text.draw(|d| {
            d.at(v2(10.0, 10.0))
                .text(&chat, |t| t.scale(12.0).color(v4(1.0, 1.0, 1.0, 1.0)))
        });

        if m.mixxx.decks[0].active {
            self.text.draw(|d| {
                d.at(v2(10.0, 1080.0 - 80.0))
                    .text(&m.mixxx.decks[0].song, |t| t.scale(12.0).color(v4(1.0, 1.0, 1.0, 1.0)))
            })
        }
        if m.mixxx.decks[1].active {
            self.text.draw(|d| {
                d.at(v2(10.0, 1080.0 - 100.0))
                    .text(&m.mixxx.decks[1].song, |t| t.scale(12.0).color(v4(1.0, 1.0, 1.0, 1.0)))
            })
        }

        self.animator.update(m.t, &mut self.scene);
    }

    pub fn view(&mut self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        self.uniform.upload(frame);

        self.scene.update(frame);
        self.psych.encode(frame, self.composite.view(0));
        self.phong.encode_load(frame, &self.scene, &self.depth, self.composite.view(0));
        self.text.encode(frame, self.composite.view(0));

        self.composite.encode(frame, target);
    }
}