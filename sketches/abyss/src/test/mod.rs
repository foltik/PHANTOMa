use lib::prelude::*;
use lib::midi2::Midi;
use lib::midi2::device::launchpad_x::{self, LaunchpadX};
use lib::midi2::device::worlde_easycontrol9;
use lib::time::DecayEnv;
use lib::gfx::pass::*;

use launchpad_x::{Output, types::Color};
use palette::{Hsv, Srgb, FromColor};
use rand::prelude::*;

use crate::{Time, StencilPass};
use crate::pipeline::*;

const N: usize = StencilPass::N;

#[macro_use]
mod macros;
tests!{
    Off      => (0, 0), Color::White,
    Test     => (1, 0), Color::Cyan,
    Uv       => (2, 0), Color::Magenta,
    UvAll    => (3, 0), Color::Magenta,
    UvScroll => (4, 0), Color::Magenta,
    Hue      => (5, 0), Color::Red,
    Colors   => (6, 0), Color::Pink,
    Whites   => (7, 0), Color::White,

    StripesInner => (0, 1), Color::Blue,
    StripesPipes => (1, 1), Color::Blue,
    StripesLight => (2, 1), Color::Blue, 
    VhsPipes     => (3, 1), Color::Blue,

    Loading  => (0, 2), Color::Orange,
    Bubbles  => (1, 2), Color::Orange,
    IsoTri   => (2, 2), Color::Orange,
    LineWave => (3, 2), Color::Orange,
    Torus    => (4, 2), Color::Orange,
    Wormhole => (5, 2), Color::Orange,

    Tiles  => (0, 3), Color::Lime,
    Primes => (1, 3), Color::Lime,
    Stars  => (2, 3), Color::Lime,
    Code   => (3, 3), Color::Lime,
}

pub struct Tester {
    mode: Test,
    initialized: bool,

    time: Time,
    decay: DecayEnv,

    stencil: StencilPass,

    off: SynthPass,
    test: TestPass,
    invert: FilterPass,
    invert_u: UniformStorage<f32>,
    abberation: FilterPass,
    abberation_u: UniformStorage<(f32, f32)>,
    alpha: FilterPass,
    alpha_u: UniformStorage<f32>,
    uv: SynthPass,
    uv_i: usize,
    whites: Vec<ColorPass>,
    whites_n: usize,
    hue: ColorPass,
    colors: Vec<ColorPass>,

    stripes_inner: StripesPass,
    stripes_inner_scroll_l: ScrollPass,
    stripes_inner_scroll_r: ScrollPass,
    stripes_pipe: StripesPass,
    stripes_pipe_scroll: ScrollPass,
    stripes_light: StripesPass,
    stripes_light_scroll: ScrollPass,

    loading: LoadingPass,
    bubbles: BubblesPass,
    isotri: IsoTriPass,
    linewave: LineWavePass,
    torus: TorusPass,
    wormhole: WormholePass,

    vhs: VhsBlocksPass,
    tiles: Vec<TilesPass>,
    primes: Vec<PrimesPass>,
    stars: Vec<StarfieldPass>,
    code: Vec<CodeScrollPass>,
}

impl Tester {
    pub fn new(device: &wgpu::Device) -> Self {
        let decay = DecayEnv::default()
            .with("beat", 250.0);

        let abberation_u = UniformStorage::new(device, "abberation", (0.0, 0.0));
        let abberation = FilterPass::new(device, "abberation", "abberation.frag.spv", Some(&abberation_u.uniform));

        let invert_u = UniformStorage::new(device, "invert", 0.0);
        let invert = FilterPass::new(device, "invert", "invert.frag.spv", Some(&invert_u.uniform));

        let alpha_u = UniformStorage::new(device, "alpha", 1.0);
        let alpha = FilterPass::new(device, "alpha", "alpha.frag.spv", Some(&alpha_u.uniform));


        let s = StencilPass::new(device);
        
        let (stripes_inner, stripes_inner_scroll_l, stripes_inner_scroll_r) = {
            let (inner_w, inner_h) = s.size("sl_inner1");
            let (w, h) = ((inner_w as f32 * 5.0) as usize, inner_h);
            let x_scale = w as f32 / inner_w as f32;

            let stripes = StripesPass::new(device, (w, h), Stripes {
                color: [1.0, 1.0, 1.0],
                n: 1,
                dir: 1,
                duty: 0.1,
                fade_l: 1.0,
                fade_r: 1.0,
            });

            let l = ScrollPass::new(device, (inner_w, inner_h), ScrollDir::Left, x_scale, 3.0);
            let r = ScrollPass::new(device, (inner_w, inner_h), ScrollDir::Right, x_scale, 3.0);
            (stripes, l, r)
        };

        let light_sz = s.size("lights");
        let stripes_light = StripesPass::new(device, light_sz, Stripes {
            color: [1.0, 1.0, 1.0],
            n: 1,
            dir: 1,
            duty: 0.5,
            fade_l: 0.0,
            fade_r: 0.0,
        });
        let stripes_light_scroll = ScrollPass::new(device, light_sz, ScrollDir::Right, 1.0, 3.0);

        let pipe_sz = s.size("sr_pipe1");
        let stripes_pipe = StripesPass::new(device, pipe_sz, Stripes {
            color: [1.0, 1.0, 1.0],
            n: 4,
            dir: 0,
            duty: 0.1,
            fade_l: 0.0,
            fade_r: 0.0,
        });
        let stripes_pipe_scroll = ScrollPass::new(device, pipe_sz, ScrollDir::Down, 1.0, 3.0);

        let tiles = s.keys().map(|k| TilesPass::new(device, s.size(k), Tiles {
            color: [1.0, 1.0, 1.0],
            nx: (s.size(k).0 as f32 / 40.0).floor(),
            ny: (s.size(k).1 as f32 / 40.0).floor(),
            t: 0.0,
        })).collect();
        let primes = s.keys().map(|k| PrimesPass::new(device, Primes {
            color: [1.0, 1.0, 1.0],
            t: 0.0,
            nx: (s.size(k).0 as f32 / 3.0).floor(),
            ny: (s.size(k).1 as f32 / 3.0).floor(),
            dx: 20.0,
            dy: 20.0,
            twin: 0.0,
            op: 0,
        })).collect();
        let stars = s.keys().map(|k| StarfieldPass::new(device, Starfield {
            color: [1.0, 1.0, 1.0],
            x: 0.5,
            y: 0.2,
            w: s.size(k).0 as f32,
            h: s.size(k).1 as f32,
            t: 0.0,
            speed: 1.0,
            warp: 0.2,
        })).collect();
        let code = s.keys().map(|k| {
            let (w, h) = s.size(k);
            CodeScrollPass::new(
                device, 
                (w, h), 
                1.0, 
                lib::resource::read_str("code1.txt"), 
                8.0, 
                ScrollDir::Down, 2.0
            )
        }).collect();

        Self {
            mode: Test::Off,
            initialized: false,

            time: Time::new(),
            decay,

            stencil: s,

            off: SynthPass::new::<()>(device, "off", "black.frag.spv", None),
            test: TestPass::new(device),
            invert,
            invert_u,
            abberation,
            abberation_u,
            alpha,
            alpha_u,
            uv: SynthPass::new::<()>(device, "uv", "uv.frag.spv", None),
            uv_i: 0,
            hue: ColorPass::new(device, [1.0, 0.0, 0.0]),
            colors: (0..N).map(|_| ColorPass::new(device, [1.0, 1.0, 1.0])).collect::<Vec<_>>(),
            whites: (0..N).map(|_| ColorPass::new(device, [1.0, 1.0, 1.0])).collect::<Vec<_>>(),
            whites_n: 1,

            stripes_inner,
            stripes_inner_scroll_l,
            stripes_inner_scroll_r,
            stripes_light,
            stripes_light_scroll,
            stripes_pipe,
            stripes_pipe_scroll,

            loading: LoadingPass::new(device),
            bubbles: BubblesPass::new(device, Bubbles {
                color: [0.0, 0.0, 0.0],
                t: 0.0,
                w: 640.0,
                h: 360.0,
                dx: 0.0,
                freq: 0.0,
            }),
            isotri: IsoTriPass::new(device, IsoTri {
                color: [1.0, 1.0, 1.0],
                aspect: 9.0 / 16.0,
                t: 0.0,
                r: 1.0,
                weight: 0.5,
                thickness: 0.5,
            }),
            linewave: LineWavePass::new(device, LineWave {
                color: [1.0, 1.0, 1.0],
                t: 0.0,
                w: 640.0,
                h: 360.0,
                n1: 0.6,
                n2: 0.25,
                dz: 0.05,
                thickness: 0.4,
                falloff: 0.0,
                n: 48,
            }),
            torus: TorusPass::new(device, Torus {
                color: [1.0, 1.0, 1.0],
                w: 640.0,
                h: 360.0,
                t: 0.0,
                fov: 0.75,
                r: 1.0,
                glow: 0.5,
                thickness: 0.0,
            }),
            wormhole: WormholePass::new(device, Wormhole {
                color: [1.0, 1.0, 1.0],
                t: 0.0,
                speed: 1.0,
                mx: 9.0 / 16.0,
                my: 1.0,
                warp: 0.5,
            }),

            vhs: VhsBlocksPass::new(device),
            tiles,
            primes,
            stars,
            code,
        }
    }

    pub async fn update(&mut self, pad: Option<&Midi<LaunchpadX>>, dt: f32) {
        self.time.update(dt, None);
        let t = self.time.t();

        self.abberation_u.0 = t;

        match self.mode {
            Test::Test => self.test.update(t),
            Test::StripesInner => {
                self.stripes_inner_scroll_l.update(t);
                self.stripes_inner_scroll_r.update(t);
            },
            Test::StripesPipes => self.stripes_pipe_scroll.update(t),
            Test::StripesLight => self.stripes_light_scroll.update(t),
            Test::VhsPipes => self.vhs.update(t),
            
            Test::Loading => self.loading.update(t),
            Test::Bubbles => self.bubbles.update(t),
            Test::IsoTri => self.isotri.update(t),
            Test::LineWave => self.linewave.update(t),
            Test::Torus => self.torus.update(t),
            Test::Wormhole => self.wormhole.update(t),

            Test::Tiles => self.tiles.iter_mut().for_each(|tile| tile.update(t)),
            Test::Primes => self.primes.iter_mut().for_each(|prime| prime.update(t)),
            Test::Stars => self.stars.iter_mut().for_each(|star| star.update(t)),
            Test::Code => self.code.iter_mut().for_each(|code| code.update(t)),
            _ => {},
        }
        if !self.initialized {
            if let Some(pad) = pad.as_ref() {
                for test in Test::all() {
                    pad.send(Output::Light(test.pos(), test.color())).await
                }
            }
            self.initialized = true;
        }
    }


    pub fn encode(&self, frame: &mut Frame, out: &wgpu::RawTextureView) {
        for v in self.stencil.all_views() {
            self.off.encode(frame, v);
        }

        let view = self.invert.view(0);

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

            Test::Hue => {
                for v in self.stencil.all_views() {
                    self.hue.encode(frame, v);
                }
                self.stencil.encode(frame, view);
            }
            Test::Colors => {
                for (v, color) in self.stencil.all_views().zip(self.colors.iter()) {
                    color.encode(frame, v);
                }
                self.stencil.encode(frame, view);
            }
            Test::Whites => {
                for (v, white) in self.stencil.all_views().zip(self.whites.iter()) {
                    white.encode(frame, v);
                }
                self.stencil.encode(frame, view);
            }

            Test::StripesInner => {
                self.stripes_inner.encode(frame, self.stripes_inner_scroll_l.view());
                self.stripes_inner.encode(frame, self.stripes_inner_scroll_r.view());
                for v in self.stencil.sl_inner() {
                    self.stripes_inner_scroll_l.encode(frame, v);
                }
                for v in self.stencil.sr_inner() {
                    self.stripes_inner_scroll_r.encode(frame, v);
                }
                self.stencil.encode(frame, view);
            }
            Test::StripesLight => {
                self.stripes_light.encode(frame, self.stripes_light_scroll.view());
                self.stripes_light_scroll.encode(frame, self.stencil.lights());
                self.stencil.encode(frame, view);
            }
            Test::StripesPipes => {
                self.stripes_pipe.encode(frame, self.stripes_pipe_scroll.view());
                for (vl, vr) in self.stencil.sl_pipes().zip(self.stencil.sr_pipes()) {
                    self.stripes_pipe_scroll.encode(frame, vl);
                    self.stripes_pipe_scroll.encode(frame, vr);
                }
                self.stencil.encode(frame, view);
            }
            Test::VhsPipes => {
                for v in self.stencil.all_views() {
                    self.vhs.encode(frame, v);
                }
                self.stencil.encode(frame, view);
            }

            Test::Loading => {
                self.loading.encode(frame, self.stencil.center());
                self.stencil.encode(frame, view);
            }
            Test::Bubbles => {
                self.bubbles.encode(frame, self.stencil.center());
                self.stencil.encode(frame, view);
            }
            Test::IsoTri => {
                self.isotri.encode(frame, self.stencil.center());
                self.stencil.encode(frame, view);
            }
            Test::LineWave => {
                self.linewave.encode(frame, self.stencil.center());
                self.stencil.encode(frame, view);
            }
            Test::Torus => {
                self.torus.encode(frame, self.stencil.center());
                self.stencil.encode(frame, view);
            }
            Test::Wormhole => {
                self.wormhole.encode(frame, self.stencil.center());
                self.stencil.encode(frame, view);
            }

            Test::Tiles => {
                for (v, e) in self.stencil.all_views().zip(self.tiles.iter()) {
                    e.encode(frame, v);
                }
                self.stencil.encode(frame, view);
            },
            Test::Primes => {
                for (v, e) in self.stencil.all_views().zip(self.primes.iter()) {
                    e.encode(frame, v);
                }
                self.stencil.encode(frame, view);
            },
            Test::Stars => {
                for (v, e) in self.stencil.all_views().zip(self.stars.iter()) {
                    e.encode(frame, v);
                }
                self.stencil.encode(frame, view);
            },
            Test::Code => {
                for (v, e) in self.stencil.all_views().zip(self.code.iter()) {
                    e.encode(frame, v);
                }
                self.stencil.encode(frame, view);
            },
        }

        self.invert_u.upload(frame);
        self.invert.encode(frame, self.abberation.view(0));

        self.abberation_u.upload(frame);
        self.abberation.encode(frame, self.alpha.view(0));

        self.alpha_u.upload(frame);
        self.alpha.encode(frame, out);
    }


    pub async fn ctrl(&mut self, input: worlde_easycontrol9::Input) {
        use worlde_easycontrol9::Input;
        log::info!("ctrl: {:?}", input);

        if let Input::Slider(8, f) = input {
            *self.alpha_u = f
        }
        if let Input::Slider(7, f) = input {
            self.abberation_u.1 = f
        }
        if let Input::Slider(6, f) = input {
            *self.invert_u = f
        }

        match self.mode {
            Test::Hue => if let Input::Slider(0, f) = input {
                let hsv = Hsv::new(f * 360.0, 1.0, 1.0);
                let (r, g, b) = Srgb::from_color(hsv).into_components();
                self.hue.color(&[r, g, b]);
            },
            Test::Whites => if let Input::Slider(0, f) = input {
                self.whites_n = (10.0 * f).floor() as usize;
            },
            Test::StripesInner | Test::StripesLight | Test::StripesPipes  => match input {
                Input::Slider(0, f) => {
                    self.stripes_inner.n = (f * 8.0).floor() as u32;
                    self.stripes_light.n = (f * 8.0).floor() as u32;
                    self.stripes_pipe.n = (f * 8.0).floor() as u32;
                },
                Input::Slider(1, f) => {
                    self.stripes_inner.duty = f;
                    self.stripes_light.duty = f;
                    self.stripes_pipe.duty = f;
                },
                Input::Slider(2, f) => {
                    self.stripes_inner.fade_l = f;
                    self.stripes_light.fade_l = f;
                    self.stripes_pipe.fade_l = f;
                },
                Input::Slider(3, f) => {
                    self.stripes_inner.fade_r = f;
                    self.stripes_light.fade_r = f;
                    self.stripes_pipe.fade_r = f;
                }
                _ => {}
            },

            Test::Bubbles => match input {
                Input::Slider(0, f) => self.bubbles.dx = f,
                Input::Slider(1, f) => self.bubbles.freq = f,
                _ => {}
            }
            Test::IsoTri => match input {
                Input::Slider(0, f) => self.isotri.r = f,
                Input::Slider(1, f) => self.isotri.weight = f,
                Input::Slider(2, f) => self.isotri.thickness = f,
                _ => {}
            }
            Test::LineWave => match input {
                Input::Slider(0, f) => self.linewave.n = (f * 100.0).floor() as u32,
                Input::Slider(1, f) => self.linewave.dz = f,
                Input::Slider(2, f) => self.linewave.falloff = f,
                Input::Slider(3, f) => self.linewave.thickness = f,
                Input::Slider(4, f) => self.linewave.n1 = f,
                Input::Slider(5, f) => self.linewave.n2 = f,
                _ => {}
            }
            Test::Torus => match input {
                Input::Slider(0, f) => self.torus.thickness = f,
                Input::Slider(1, f) => self.torus.glow = f,
                Input::Slider(2, f) => self.torus.fov = f,
                Input::Slider(3, f) => self.torus.r = f,
                _ => {}
            }
            Test::Wormhole => match input {
                Input::Slider(0, f) => self.wormhole.speed = f,
                Input::Slider(1, f) => self.wormhole.warp = f,
                _ => {},
            }
            
            Test::Primes => match input {
                Input::Slider(0, f) => self.primes.iter_mut().for_each(|p| p.op = (f * 2.0).floor() as u32),
                Input::Slider(1, f) => self.primes.iter_mut().for_each(|p| p.twin = f),
                Input::Slider(2, f) => self.primes.iter_mut().for_each(|p| p.dx = 100.0 * f),
                Input::Slider(3, f) => self.primes.iter_mut().for_each(|p| p.dy = 100.0 * f),
                _ => {}
            },
            Test::Stars => match input {
                Input::Slider(0, f) => self.stars.iter_mut().for_each(|s| s.speed = f),
                Input::Slider(1, f) => self.stars.iter_mut().for_each(|s| s.warp = f),
                Input::Slider(2, f) => self.stars.iter_mut().for_each(|s| s.x = f),
                Input::Slider(3, f) => self.stars.iter_mut().for_each(|s| s.y = f),
                _ => {}
            },
            _ => {},
        }
    }

    pub async fn pad(&mut self, input: launchpad_x::Input) {
        log::info!("pad: {:?}", input);

        use launchpad_x::{*, types::*};
        if let Input::Press(i, _) = input {
            let p = Pos::from(i);

            let t = self.time.t();

            let mode = Test::all().find(|t| t.pos() == p);
            if let Some(mode) = mode {
                self.mode = mode;
                match mode {
                    Test::UvScroll => {
                        self.uv_i = (self.uv_i + 1) % N;
                    },
                    Test::Colors => {
                        let mut rng = rand::thread_rng();
                        for color in &mut self.colors {
                            let hsv = Hsv::new(rng.gen::<f32>() * 360.0, 1.0, 1.0);
                            let (r, g, b) = Srgb::from_color(hsv).into_components();
                            color.color(&[r, g, b]);
                        }
                    },
                    Test::Whites => {
                        let mut rng = rand::thread_rng();

                        self.whites.iter_mut().for_each(|w| w.color(&[0.0, 0.0, 0.0]));
                        self.whites.iter_mut()
                            .choose_multiple(&mut rng, self.whites_n)
                            .iter_mut()
                            .for_each(|w| w.color(&[1.0, 1.0, 1.0]));
                    },

                    Test::StripesInner => {
                        self.stripes_inner_scroll_l.start_now(t);
                        self.stripes_inner_scroll_r.start_now(t);
                    },
                    Test::StripesLight => self.stripes_light_scroll.start_now(t),
                    Test::StripesPipes => self.stripes_pipe_scroll.start_now(t),

                    Test::Code => {
                        for c in &mut self.code {
                            c.start(t);
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    pub async fn key(&mut self, key: Key) {
        log::info!("key: {:?}", key);
    }
}