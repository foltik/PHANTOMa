use lib::audio::Midi;

use lib::gfx::frame::Frame;
use lib::gfx::pass::FilterPass;
use lib::gfx::uniform::UniformStorage;
use lib::gfx::wgpu;

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct FxState {
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

pub struct Fx {
    state: UniformStorage<FxState>,

    edge: FilterPass,
    shake: FilterPass,
    glitch: FilterPass,
    vhs: FilterPass,
    fade: FilterPass,
    pause: FilterPass,
}

impl Fx {
    pub fn new(device: &wgpu::Device) -> Self {
        let state = UniformStorage::new(device, "fx", FxState::default());

        let edge = FilterPass::new(device, "edge", "edge.frag.spv", Some(state.as_ref()));
        let shake = FilterPass::new(device, "shake", "shake.frag.spv", Some(state.as_ref()));
        let glitch = FilterPass::new(device, "glitch", "glitch.frag.spv", Some(state.as_ref()));
        let vhs = FilterPass::new(device, "vhs", "vhs.frag.spv", Some(state.as_ref()));
        let fade = FilterPass::new(device, "fade", "fade.frag.spv", Some(state.as_ref()));
        let pause = FilterPass::new(device, "pause", "pause.frag.spv", Some(state.as_ref()));

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

    pub fn update(&mut self, tc: f32, t: f32) {
        self.state.tc = tc;
        self.state.t = t;
    }

    pub fn midi(&mut self, msg: Midi) {
        match msg {
            Midi::Slider(0, f) => self.state.edge = f,
            Midi::Slider(1, f) => self.state.glitch = f,
            Midi::Slider(2, f) => self.state.vhs = f,
            Midi::Slider(3, f) => self.state.pause = f,
            Midi::Slider(4, f) => self.state.black = f,
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

impl std::ops::Deref for Fx {
    type Target = FxState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl std::ops::DerefMut for Fx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
