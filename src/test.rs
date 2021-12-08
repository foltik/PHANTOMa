use lib::prelude::*;
use lib::midi2::device::launchpad_x;
use lib::midi2::device::worlde_easycontrol9;
use lib::time::DecayEnv;
use lib::gfx::pass::*;

use palette::{Hsv, Srgb, FromColor};
use rand::prelude::*;

use crate::{Model, StencilPass};
use crate::pipeline::*;

use launchpad_x::types::Color;
self::tests!{
    Off      => (0, 0), Color::White,
    Test     => (0, 1), Color::Orange,

    Uv       => (1, 0), Color::Magenta,
    UvAll    => (2, 0), Color::Magenta,
    UvScroll => (3, 0), Color::Magenta,

    Hue      => (4, 0), Color::Red,
    Colors   => (5, 0), Color::Pink
}

pub struct Tester {
    mode: Test,
    decay: DecayEnv,
    initialized: bool,

    stencil: StencilPass,

    off: SynthPass,
    alpha: FilterPass,
    alpha_u: UniformStorage<f32>,
    test: TestPass,
    uv: SynthPass,
    uv_i: usize,
    colors: Vec<ColorPass>,
    hue: ColorPass,
}

impl Tester {
    pub fn new(device: &wgpu::Device) -> Self {
        let decay = DecayEnv::default()
            .with("beat", 250.0);

        let alpha_u = UniformStorage::new(device, "alpha", 1.0);
        let alpha = FilterPass::new::<f32>(device, "alpha", "alpha.frag.spv", Some(&alpha_u.uniform));

        Self {
            mode: Test::Off,
            decay,
            initialized: false,

            stencil: StencilPass::new(device),

            off: SynthPass::new::<()>(device, "off", "black.frag.spv", None),
            alpha,
            alpha_u,
            test: TestPass::new(device),
            uv: SynthPass::new::<()>(device, "uv", "uv.frag.spv", None),
            uv_i: 0,
            colors: (0..29).map(|_| ColorPass::new(device, [1.0, 1.0, 1.0])).collect::<Vec<_>>(),
            hue: ColorPass::new(device, [1.0, 0.0, 0.0]),
        }
    }


    pub fn update(mut self, m: &mut Model, dt: f32) -> Self {
        match self.mode {
            Test::Test => self.test.update(m.tc()),
            _ => {},
        }
        if !self.initialized {
            if let Some(pad) = m.pad.as_ref() {
                use launchpad_x::{*, types::*};
                pad.send(Output::Light(Coord(0, 0).into(), Color::White));
                pad.send(Output::Light(Coord(0, 1).into(), Color::Orange));
                pad.send(Output::Light(Coord(1, 0).into(), Color::Magenta));
                pad.send(Output::Light(Coord(2, 0).into(), Color::Magenta));
                pad.send(Output::Light(Coord(3, 0).into(), Color::Magenta));
                pad.send(Output::Light(Coord(4, 0).into(), Color::Red));
                pad.send(Output::Light(Coord(5, 0).into(), Color::Pink));
            }
        }
        self
    }


    pub fn encode(self, frame: &mut Frame, out: &wgpu::RawTextureView) -> Self {
        let view = self.alpha.view(0);
        match self.mode {
            Test::Off => self.off.encode(frame, view),
            Test::Test => self.test.encode(frame, view),

            Test::Uv => self.uv.encode(frame, view),
            Test::UvAll => {
                for v in self.stencil.all_views() {
                    self.uv.encode(frame, v);
                }
                self.stencil.encode(frame, view);
            },
            Test::UvScroll => {
                let v = self.stencil.all_views().nth(self.uv_i).unwrap();
                self.uv.encode(frame, v);
                self.stencil.encode(frame, view);

            }
            Test::Colors => {
                for (v, color) in self.stencil.all_views().zip(self.colors.iter()) {
                    color.encode(frame, v);
                }
                self.stencil.encode(frame, view);
            }
            Test::Hue => {
                for v in self.stencil.all_views() {
                    self.hue.encode(frame, v);
                }
                self.stencil.encode(frame, view);
            }
        }
        self.alpha_u.upload(frame);
        self.alpha.encode(frame, out);
        self
    }


    pub fn ctrl(mut self, m: &mut crate::Model, input: worlde_easycontrol9::Input) -> Self {
        log::info!("ctrl: {:?}", input);

        use worlde_easycontrol9::Input;
        match input {
            Input::Slider(0, f) => *self.alpha_u = f,
            Input::Slider(1, f) => {
                let hsv = Hsv::new(f * 360.0, 1.0, 1.0);
                let (r, g, b) = Srgb::from_color(hsv).into_components();
                self.hue.color(&[r, g, b]);
            },
            _ => {}
        }
        self
    }

    pub fn pad(mut self, m: &mut Model, input: launchpad_x::Input) -> Self {
        log::info!("pad: {:?}", input);

        use launchpad_x::{*, types::*};
        if let Input::Press(i, _) = input {
            let p = Pos::from(i);
            let Coord(x, y) = p.into();

            match (x, y) {
                (5, 0) => {
                    let mut rng = rand::thread_rng();
                    for color in &mut self.colors {
                        let hsv = Hsv::new(rng.gen::<f32>() * 360.0, 1.0, 1.0);
                        let (r, g, b) = Srgb::from_color(hsv).into_components();
                        color.color(&[r, g, b]);
                    }
                },
                _ => {}
            }

            if let Some(mode) = match (x, y) {
                (0, 0) => Some(Test::Off),
                (1, 0) => Some(Test::Test),
                (3, 0) => Some(Test::Uv),
                (4, 0) => Some(Test::UvAll),
                (5, 0) => Some(Test::Colors),
                (6, 0) => Some(Test::Hue),
                _ => None
            } {
                self.mode = mode;
            }
        }
        self
    }

    pub fn key(mut self, m: &mut Model, key: Key) -> Self {
        log::info!("key: {:?}", key);
        self
    }
}

macro_rules! tests {
    {$($Variant:ident => ($X:literal, $Y:literal), $Color:expr),*} =>
    {
        #[derive(Debug)]
        pub enum Test {
            $($Variant),*
        }

        impl Test {
            pub fn xy(&self) -> (i8, i8) {
                match *self {
                    $(Test::$Variant => ($X, $Y)),*
                }
            }

            pub fn color(&self) -> launchpad_x::types::Color {
                match *self {
                    $(Test::$Variant => $Color),*
                }
            }

            pub fn pos(&self) -> launchpad_x::types::Pos {
                use launchpad_x::types::{Pos, Coord};
                let (x, y) = self.xy();
                Pos::from(Coord(x, y))
            }
        }

        const TESTS: &'static [Test] = &[$(Test::$Variant),*];
    }
}