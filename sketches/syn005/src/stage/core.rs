use lib::prelude::*;

use lib::resource;
use gfx::scene::Scene;
use gfx::animation::Animator;
use gfx::pass::{FilterPass, RingPass, TextPass, TextPassBuilder, ImagePass};
use lib::twitch::TwitchBuffer;

use crate::Model;
use crate::pipeline;

pub struct Core {
    init: bool,

    scene: Scene,
    animator: Animator,

    phong: pipeline::Phong,
    text: TextPass,
    flip: FilterPass,
    depth: wgpu::TextureView,
}

impl Core {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let scene = resource::read_scene(device, "core2.glb");

        let animator = Animator::new(&scene);

        let phong = pipeline::Phong::new(app, &scene, |name| true, |mat| true);
        let text = TextPassBuilder::new()
            .with("default", "go-b.ttf")
            .build(device);

        let flip = FilterPass::new::<()>(device, "flip", "passthrough.frag.spv", None);

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
            flip,
            depth,
        }
    }

    pub fn input(&mut self, app: &App, m: &mut Model, state: KeyState, key: Key) {

    }

    pub fn midi(&mut self, m: &mut Model, _bank: MidiBank, msg: Midi) {

    }

    fn init(&mut self, app: &App, m: &mut Model, dt: f32) {
        self.animator.play(m.t, true, "Action.001");

        self.animator.play(m.t, true, "CoreAction");
        self.animator.play(m.t, true, "Ring 1Action");
        self.animator.play(m.t, true, "Ring 2Action");
        self.animator.play(m.t, true, "Ring 3Action.001");
        self.animator.play(m.t, true, "Ring 3Action.002");
    }

    pub fn update(&mut self, app: &App, m: &mut Model, dt: f32) {
        if !self.init {
            self.init(app, m, dt);
            self.init = true;
        }

        let chat = m
            .twitch
            .block(120, 20)
            .join("\n");
        self.text.draw(|d| {
            d.at(v2(10.0, 10.0))
                .text(&chat, |t| t.scale(12.0).color(v4(1.0, 1.0, 1.0, 0.1)))
        });

        if m.mixxx.decks[0].active {
            self.text.draw(|d| {
                d.at(v2(10.0, 1080.0 - 80.0))
                    .text(&m.mixxx.decks[0].song, |t| t.scale(12.0).color(v4(1.0, 1.0, 1.0, 0.1)))
            })
        }
        if m.mixxx.decks[1].active {
            self.text.draw(|d| {
                d.at(v2(10.0, 1080.0 - 100.0))
                    .text(&m.mixxx.decks[1].song, |t| t.scale(12.0).color(v4(1.0, 1.0, 1.0, 0.1)))
            })
        }

        self.animator.update(m.t, &mut self.scene);

        let cam = &mut self.scene.desc.nodes[self.scene.desc.names["Camera"]].transform;

        let h = 2.5 + (m.t * 2.0).sin() * 1.1;
        cam.translate.y = h;
    }

    pub fn view(&mut self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        self.scene.update(frame);
        self.phong.encode(frame, &self.scene, &self.depth, self.flip.view(0));
        self.text.encode(frame, self.flip.view(0));

        self.flip.encode(frame, target);
    }
}