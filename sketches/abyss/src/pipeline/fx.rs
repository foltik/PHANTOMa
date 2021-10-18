use lib::gfx::frame::Frame;
use lib::gfx::pass::FilterPass;
use lib::gfx::uniform::UniformStorage;
use lib::gfx::wgpu;

use lib::midi2::device::worlde_easycontrol9::Input;

use crate::Time;

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct Fx {
    t: f32,
    tc: f32,
    pause: f32,
    glitch: f32,
    glitch_mo: f32,
    vhs: f32,
    red: f32,
    flash: f32,
    shake: f32,
    black: f32,
    edge: f32,
    mega: f32,
}

pub struct FxPass {
    state: UniformStorage<Fx>,

    edge: FilterPass,
    shake: FilterPass,
    glitch: FilterPass,
    vhs: FilterPass,
    fade: FilterPass,
    pause: FilterPass,
}

impl FxPass {
    pub fn new(device: &wgpu::Device, size: (usize, usize)) -> Self {
        let state = UniformStorage::new(device, "fx", Fx::default());

        let edge = FilterPass::new_sized(device, "edge", "edge.frag.spv", Some(state.as_ref()), size);
        let shake = FilterPass::new_sized(device, "shake", "shake.frag.spv", Some(state.as_ref()), size);
        let glitch = FilterPass::new_sized(device, "glitch", "glitch.frag.spv", Some(state.as_ref()), size);
        let vhs = FilterPass::new_sized(device, "vhs", "vhs.frag.spv", Some(state.as_ref()), size);
        let fade = FilterPass::new_sized(device, "fade", "fade.frag.spv", Some(state.as_ref()), size);
        let pause = FilterPass::new_sized(device, "pause", "pause.frag.spv", Some(state.as_ref()), size);

        Self {
            state,
            edge,
            shake,
            glitch,
            vhs,
            fade,
            pause,
        }
    }

    pub fn update(&mut self, time: &Time) {
        self.state.t = time.t_rms();
        self.state.tc = time.t();
    }

    pub fn ctrl(&mut self, input: Input) {
        match input {
            Input::Slider(0, f) => self.state.edge = f,
            Input::Slider(1, f) => self.state.glitch = f,
            Input::Slider(2, f) => self.state.vhs = f,
            Input::Slider(3, f) => self.state.pause = f,
            Input::Slider(4, f) => self.state.black = f,
            _ => {},
        }
    }

    pub fn view(&self) -> &wgpu::RawTextureView {
        self.edge.view(0)
    }

    pub fn upload(&self, frame: &mut Frame) {
        self.state.upload(frame);
    }

    pub fn encode(&self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        self.edge.encode(frame, self.shake.view(0));
        self.shake.encode(frame, self.glitch.view(0));
        self.glitch.encode(frame, self.vhs.view(0));
        self.vhs.encode(frame, self.fade.view(0));
        self.fade.encode(frame, self.pause.view(0));
        self.pause.encode(frame, target);
    }
}

impl std::ops::Deref for FxPass {
    type Target = Fx;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl std::ops::DerefMut for FxPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
