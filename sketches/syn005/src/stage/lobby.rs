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

pub struct Lobby {
    init: bool,

    scene: Scene,
    animator: Animator,

    phong: pipeline::Phong,
    images: ImagePass,
    text: TextPass,
    composite: FilterPass,
    depth: wgpu::TextureView,
}

impl Lobby {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let scene = resource::read_scene(device, "lobby.glb");
        let animator = Animator::new(&scene);

        let phong = pipeline::Phong::new(app, &scene, |name| true, |mat| true);
        let images = ImagePass::new(device)
            .with(app, "ss2title.png", "cockpit", v2(0.0, 0.0), v2(1.0, 1.0));
        let text = TextPassBuilder::new()
            .with("default", "go-b.ttf")
            .with("italic", "go-bi.ttf")
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
        self.animator.play(m.t, true, "PsyAction");
    }

    pub fn update(&mut self, app: &App, m: &mut Model, dt: f32) {
        if !self.init {
            self.init(app, m, dt);
            self.init = true;
        }

        let chat = m
            .twitch
            .block(120, 16)
            .join("\n");
        self.text.draw(|d| {
            d.at(v2(10.0, 10.0))
                .text(&chat, |t| t.scale(12.0).color(v4(1.0, 1.0, 1.0, 1.0)))
        });

        self.animator.update(m.t, &mut self.scene);

        // let h = m.t.sin() * 0.1;
        // self.scene.desc.nodes[self.scene.desc.names["Psy"]].transform.translate.y = h;
    }

    pub fn view(&mut self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        self.scene.update(frame);
        self.images.encode(frame, self.composite.view(0));
        self.phong.encode(frame, &self.scene, &self.depth, self.composite.view(1));
        self.text.encode(frame, self.composite.view(1));

        self.composite.encode(frame, target);
    }
}