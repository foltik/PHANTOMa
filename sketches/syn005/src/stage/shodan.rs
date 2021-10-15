use lib::prelude::*;

use lib::resource;
use gfx::scene::Scene;
use gfx::animation::Animator;
use gfx::pass::{FilterPass, RingPass, TextPass, TextPassBuilder, ImagePass};
use lib::twitch::TwitchBuffer;

use crate::Model;
use crate::pipeline;

pub struct Shodan {
    init: bool,

    scene: Scene,
    animator: Animator,

    phong: pipeline::Phong,
    text: TextPass,
    // animated: pipeline::Animated,
    // images: ImagePass,
    composite: FilterPass,
    depth: wgpu::TextureView,
}

impl Shodan {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let scene = resource::read_scene(device, "face.glb");

        let animator = Animator::new(&scene);

        let phong = pipeline::Phong::new(app, &scene, |name| true, |mat| true);
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

            phong,
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
        self.animator.play(m.t, true, "Action");
    }

    pub fn update(&mut self, app: &App, m: &mut Model, dt: f32) {
        if !self.init {
            self.init(app, m, dt);
            self.init = true;
        }

        if m.beat_v {
            self.animator.play(m.t, false, "handleAction");
        }

        let chat = m
            .twitch
            .block(120, 20)
            .join("\n");
        self.text.draw(|d| {
            d.at(v2(10.0, 10.0))
                .text(&chat, |t| t.scale(12.0).color(v4(0.0, 1.0, 0.0, 0.1)))
        });

        if m.mixxx.decks[0].active {
            self.text.draw(|d| {
                d.at(v2(10.0, 1080.0 - 80.0))
                    .text(&m.mixxx.decks[0].song, |t| t.scale(12.0).color(v4(0.0, 1.0, 0.0, 0.1)))
            })
        }
        if m.mixxx.decks[1].active {
            self.text.draw(|d| {
                d.at(v2(10.0, 1080.0 - 100.0))
                    .text(&m.mixxx.decks[1].song, |t| t.scale(12.0).color(v4(0.0, 1.0, 0.0, 0.1)))
            })
        }

        self.animator.update(m.t, &mut self.scene);
    }

    pub fn view(&mut self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        self.scene.update(frame);
        self.phong.encode(frame, &self.scene, &self.depth, self.composite.view(0));
        self.text.encode(frame, self.composite.view(0));

        self.composite.encode(frame, target);
    }
}