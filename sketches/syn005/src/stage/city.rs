use lib::prelude::*;

use gfx::animation::Animator;
use gfx::pass::{FilterPass, ImagePass, RingPass, TextPass, TextPassBuilder};
use gfx::scene::Scene;
use lib::resource;
use lib::twitch::TwitchBuffer;

use crate::pipeline;
use crate::Model;

pub struct City {
    scene: Scene,
    animator: Animator,

    phong: pipeline::Phong,
    phong_nowin: pipeline::Phong,
    text: TextPass,
    animated: pipeline::Animated,
    composite: FilterPass,

    depth: wgpu::TextureView,
}

impl City {
    pub fn new(app: &App) -> Self {
        let device = &app.device;

        let scene = resource::read_scene(device, "city.glb");
        let animator = Animator::new(&scene);

        let phong = pipeline::Phong::new(app, &scene, |name| true, |mat| true);
        let phong_nowin =
            pipeline::Phong::new(app, &scene, |name| true, |name| !name.contains("Windows"));
        let animated = pipeline::Animated::new(app, &scene, &animations());
        let composite = FilterPass::new_composite::<()>(device, "composite", 3, None, None);

        let text = TextPassBuilder::new()
            .with("default", "go-b.ttf")
            .build(device);

        let depth = wgpu::util::TextureBuilder::new_depth("depth")
            .build(&app.device)
            .view()
            .build();

        Self {
            animator,

            scene,
            phong,
            phong_nowin,
            text,
            animated,
            composite,

            depth,
        }
    }

    pub fn input(&mut self, app: &App, m: &mut Model, state: KeyState, key: Key) {}

    pub fn midi(&mut self, m: &mut Model, _bank: MidiBank, msg: Midi) {}

    pub fn update(&mut self, app: &App, m: &mut Model, dt: f32) {
        let chat = m
            .twitch
            .block(120, 20)
            .join("\n");
        self.text.draw(|d| {
            d.at(v2(10.0, 10.0))
                .text(&chat, |t| t.scale(12.0).color(v4(0.0, 1.0, 0.0, 0.1)))
        });

        self.animated.update(m.t);
        self.animator.update(m.t, &mut self.scene);
    }

    pub fn view(&mut self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        self.scene.update(frame);
        self.animated.upload(frame);

        // self.phong.encode(frame, &self.scene, &self.depth, self.composite.view(0));
        self.phong_nowin
            .encode(frame, &self.scene, &self.depth, self.composite.view(0));
        self.animated.encode(frame, &self.scene, &self.depth, self.composite.view(1));

        self.text.encode(frame, self.composite.view(2));

        self.composite.encode(frame, target);
    }
}

fn animations() -> Vec<pipeline::AnimatedMaterialDesc> {
    let triopt = pipeline::AnimatedMaterialDesc {
        name: "TriOpt",
        // nodes: vec!["Plane"],
        nodes: vec![],
        mats: vec![
            "Windows",
            "Windows.001",
            "Windows.003",
            "Windows.019",
            "Windows.018",
            "Windows.017",
            "Windows.016",
            "Windows.015",
            "Windows.014",
            "Windows.013",
            "Windows.012",
            "Windows.010",
            "Windows.009",
            "Windows.008",
            "Windows.007",
            "Windows.020",
            "Windows.021",
            "Windows.022",
        ],
        images: vec!["00_.png", "00_1.png", "00_2.png", "00_3.png"],
        fps: 10,
        unlit: false,
    };

    vec![triopt]
}
