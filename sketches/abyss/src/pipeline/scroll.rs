use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::{FilterPass};
use lib::gfx::uniform::UniformStorage;

pub struct ScrollPass {
    input: FilterPass,
    composite: FilterPass,
    uniform: UniformStorage<ScrollState>,

    active: bool,
    start: f32,
    t: f32,
    pd: f32,
}

pub enum ScrollDir {
    Up,
    Down,
    Left,
    Right,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ScrollState {
    fr: f32,
    scale: f32,
    dir: i32,
    rev: i32,
}

impl ScrollPass {
    pub fn new(device: &wgpu::Device, size: (usize, usize), dir: ScrollDir, scale: f32, pd: f32) -> Self {
        let uniform = UniformStorage::new(device, "scroll_fr", ScrollState {
            fr: -1.0,
            scale,
            dir: match dir {
                ScrollDir::Up   | ScrollDir::Down  => 0,
                ScrollDir::Left | ScrollDir::Right => 1,
            },
            rev: match dir {
                ScrollDir::Up   | ScrollDir::Left  => -1,
                ScrollDir::Down | ScrollDir::Right => 1,
            },
        });

        let input = FilterPass::new_passthrough_sized(device, size);
        let composite = FilterPass::new_composite(device, "scroll_composite", 2, Some("scroll2.frag.spv"), Some(&uniform.uniform));

        Self {
            input,
            composite,
            uniform,

            active: false,
            start: 0.0,
            t: 0.0,
            pd,
        }
    }

    fn fr(&self, pd: f32) -> f32 {
        if self.t < pd {
            (self.t - pd) / pd
        } else {
            (self.t % pd) / pd
        }
    }

    pub fn update(&mut self, t: f32) {
        if self.active {
            self.t = t - self.start;
        }
        self.uniform.fr = self.fr(self.pd);
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.uniform.upload(frame);

        self.input.encode(frame, self.composite.view(0));
        self.input.encode(frame, self.composite.view(1));

        self.composite.encode(frame, view);
    }

    pub fn view(&self) -> &wgpu::RawTextureView {
        self.input.view(0)
    }


    pub fn start(&mut self, t: f32) {
        self.active = true;
        self.start = t;
    }

    pub fn start_now(&mut self, t: f32) {
        self.active = true;
        self.start = t - self.pd;
        self.t = 0.0;
    }

    pub fn stop(&mut self) {
        self.active = false;
        self.start = 0.0;
        self.t = 0.0;
    }

    pub fn active(&self) -> bool {
        self.active
    }


    pub fn dir(&mut self, dir: ScrollDir) {
        self.uniform.dir = match dir {
            ScrollDir::Up   | ScrollDir::Down  => 0,
            ScrollDir::Left | ScrollDir::Right => 1,
        };
        self.uniform.rev = match dir {
            ScrollDir::Up   | ScrollDir::Left  => -1,
            ScrollDir::Down | ScrollDir::Right => 1,
        };
    }

    pub fn pd(&mut self, pd: f32) {
        let fr0 = self.fr(self.pd);
        let fr1 = self.fr(pd);

        self.start += (fr1 - fr0) * pd;
        self.pd = pd;
    }
}
