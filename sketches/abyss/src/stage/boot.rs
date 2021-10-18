use lib::prelude::*;
use lib::midi2::device::launchpad_x;
use lib::midi2::device::worlde_easycontrol9;

use lib::time::Decay;

use async_trait::async_trait;

use crate::{Model, StencilPass};
use crate::pipeline::*;

use super::Render;

pub struct Boot {
    loading_active: bool,
    loading: LoadingPass,

    inner_stripes: StripesPass,
    inner_scroll_l: ScrollPass,
    inner_scroll_r: ScrollPass,

    light_stripes: StripesPass,
    light_scroll: ScrollPass,

    code_wing_l: CodeScrollPass,
    code_wing_r: CodeScrollPass,

    tiles: TilesPass,

    flash_l: ColorPass,
    flash_r: ColorPass,
    flash_l_decay: Decay,
    flash_r_decay: Decay,
}

impl Boot {
    pub fn new(device: &wgpu::Device, s: &StencilPass) -> Self {

        let (inner_stripes, inner_scroll_l, inner_scroll_r) = {
            let (inner_w, inner_h) = s.size("sl_inner1");
            let (w, h) = ((inner_w as f32 * 5.0) as usize, inner_h);
            let x_scale = w as f32 / inner_w as f32;

            let stripes = StripesPass::new(device, (w, h), Stripes {
                color: [1.0, 1.0, 1.0],
                n: 1,
                dir: 1,
                duty: 0.9,
                fade_l: 1.0,
                fade_r: 1.0,
            });

            let l = ScrollPass::new(device, (inner_w, inner_h), ScrollDir::Left, x_scale, 3.0);
            let r = ScrollPass::new(device, (inner_w, inner_h), ScrollDir::Right, x_scale, 3.0);
            (stripes, l, r)
        };

        let light_sz = s.size("lights");
        let light_stripes = StripesPass::new(device, light_sz, Stripes {
            color: [1.0, 1.0, 1.0],
            n: 1,
            dir: 1,
            duty: 0.5,
            fade_l: 0.0,
            fade_r: 0.0,
        });
        let light_scroll = ScrollPass::new(device, light_sz, ScrollDir::Right, 1.0, 3.0);

        let (wing_w, wing_h) = s.size("sl_wing");
        let code_wing_l = {
            let (w, h) = (wing_w, (wing_h as f32 * 1.0) as usize);
            let y_scale = h as f32 / wing_h as f32;
            let str = lib::resource::read_str("code0.txt");
            CodeScrollPass::new(device, (w, h), y_scale, str, 8.0, ScrollDir::Down, 2.0)
        };
        let code_wing_r = {
            let (w, h) = (wing_w, (wing_h as f32 * 1.15) as usize);
            let y_scale = h as f32 / wing_h as f32;
            let str = lib::resource::read_str("code1.txt");
            CodeScrollPass::new(device, (w, h), y_scale, str, 8.0, ScrollDir::Down, 1.6)
        };

        let pod_sz = s.size("djl_pod");
        let tiles = TilesPass::new(device, pod_sz, Tiles {
            color: [0.0, 0.0, 0.0],
            nx: 26.0,
            ny: 4.0,
            t: 0.0,
        });

        Self {
            loading_active: false,
            loading: LoadingPass::new(device),

            inner_stripes,
            inner_scroll_l,
            inner_scroll_r,

            light_stripes,
            light_scroll,

            code_wing_l,
            code_wing_r,

            tiles,

            flash_l: ColorPass::new(device, [0.0, 0.0, 0.0]),
            flash_r: ColorPass::new(device, [0.0, 0.0, 0.0]),
            flash_l_decay: Decay::new(150.0),
            flash_r_decay: Decay::new(150.0),
        }
    }
}

#[async_trait]
impl Render for Boot {
    async fn init(&mut self, _app: &App, m: &mut Model) {
        self.code_wing_l.stop();
        self.code_wing_r.stop();

        self.light_scroll.stop();

        self.inner_scroll_l.stop();
        self.inner_scroll_r.stop();

        if let Some(pad) = m.pad.as_ref() {
            use launchpad_x::{*, types::*};
            use Color as Color;

            pad.send(Output::Clear).await;

            // L/R flash
            pad.send(Output::Light(Coord(1, 6).into(), Color::White)).await;
            pad.send(Output::Light(Coord(6, 6).into(), Color::White)).await;

            // loading, inner scroll start
            pad.send(Output::Light(Coord(3, 6).into(), Color::Red)).await;

            // top lights start
            pad.send(Output::Light(Coord(3, 5).into(), Color::Red)).await;

            // L/R code scroll start
            pad.send(Output::Light(Coord(3, 4).into(), Color::Red)).await;
            pad.send(Output::Light(Coord(4, 4).into(), Color::Red)).await;

            // podium tiles
            pad.send(Output::Light(Coord(3, 3).into(), Color::Red)).await;
        }
    }


    async fn key(&mut self, _app: &App, _m: &mut Model, _key: Key) {
        // match key {
        //     _ => {}
        // }
    }

    async fn ctrl(&mut self, _app: &App, _m: &mut crate::Model, _input: worlde_easycontrol9::Input) {
        // use worlde_easycontrol9::Input;
        // match input {
        //     _ => {}
        // }
    }

    async fn pad(&mut self, _app: &App, m: &mut crate::Model, input: launchpad_x::Input) {
        use launchpad_x::{*, types::*};
        let pad = m.pad.as_ref().unwrap();

        match input {
            Input::Press(i, _) => {
                let p = Pos::from(i);
                let Coord(x, y) = p.into();

                match (x, y) {
                    // loading, inner scroll start
                    (3, 6) => {
                        self.inner_scroll_l.start_now(m.t.t_rms());
                        self.inner_scroll_r.start_now(m.t.t_rms());
                        self.loading_active = true;
                        pad.send(Output::Light(p, Color::Lime)).await;
                    },

                    // top lights start
                    (3, 5) => {
                        self.light_scroll.start(m.t());
                        pad.send(Output::Light(p, Color::Lime)).await;
                    },

                    // L/R code scroll
                    (3, 4) => {
                        self.code_wing_l.start(m.t.t_rms());
                        pad.send(Output::Light(p, Color::Lime)).await;
                    },
                    (4, 4) => {
                        self.code_wing_r.start(m.t.t_rms());
                        pad.send(Output::Light(p, Color::Lime)).await;
                    }

                    // podium tiles
                    (3, 3) => {
                        self.tiles.color = [1.0, 1.0, 1.0];
                        pad.send(Output::Light(p, Color::Lime)).await;
                    },

                    // L/R flash
                    (1, 6) => { 
                        self.flash_l_decay.hold(true);
                        self.flash_l_decay.set();
                    }
                    (6, 6) => { 
                        self.flash_r_decay.hold(true);
                        self.flash_r_decay.set();
                    }
                    _ => {}
                }
            },
            Input::Release(i) => {
                let Coord(x, y) = Pos::from(i).into();

                // log::info!("release ({}, {})", x, y);

                match (x, y) {
                    (1, 6) => {
                        self.flash_l_decay.hold(false);
                    }
                    (6, 6) => {
                        self.flash_r_decay.hold(false);
                    }
                    _ => {}
                }
            }
            _ => {},
        }
    }

    async fn update(&mut self, _app: &App, m: &mut Model, dt: f32) {
        self.loading.update(m.t());

        self.inner_scroll_l.update(m.tc());
        self.inner_scroll_r.update(m.tc());

        self.light_scroll.update(m.tc());

        self.code_wing_r.update(m.tc());
        self.code_wing_l.update(m.tc());

        self.tiles.update(m.tc());

        self.flash_l_decay.update(dt);
        self.flash_r_decay.update(dt);
        let v = self.flash_l_decay.v();
        self.flash_l.color(&[v, v, v]);
        let v = self.flash_r_decay.v();
        self.flash_r.color(&[v, v, v]);
    }


    fn view(&mut self, frame: &mut Frame, s: &StencilPass) {
        if self.loading_active {
            self.loading.encode(frame, s.center());
        }

        // FIXME: DONT NEED EVERY FRAME
        self.inner_stripes.encode(frame, self.inner_scroll_l.view());
        self.inner_stripes.encode(frame, self.inner_scroll_r.view());
        for view in s.sl_inner() {
            self.inner_scroll_l.encode(frame, view);
        }
        for view in s.sr_inner() {
            self.inner_scroll_r.encode(frame, view);
        }

        self.light_stripes.encode(frame, self.light_scroll.view());
        self.light_scroll.encode(frame, s.lights());

        self.code_wing_l.encode(frame, s.sl_wing());
        self.code_wing_r.encode(frame, s.sr_wing());

        self.tiles.encode(frame, s.djl_pod());
        self.tiles.encode(frame, s.djr_pod());

        for view in s.sl_pipes() {
            self.flash_l.encode(frame, view);
        }
        for view in s.sr_pipes() {
            self.flash_r.encode(frame, view);
        }
    }
}