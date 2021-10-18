use lib::gfx::frame::Frame;
use lib::gfx::wgpu;
use lib::gfx::pass::SynthPass;
use lib::gfx::uniform::UniformStorage;

pub struct ColorPass {
    synth: SynthPass,
    uniform: UniformStorage<[f32; 3]>,
}

impl ColorPass {
    pub fn new(device: &wgpu::Device, color: [f32; 3]) -> Self {
        let uniform = UniformStorage::new(device, "scroll_fr", color);
        let synth = SynthPass::new(device, "fill", "color.frag.spv", Some(&uniform.uniform));
        Self {
            synth,
            uniform,
        }
    }

    pub fn color(&mut self, color: &[f32; 3]) {
        self.uniform.copy_from_slice(color);
    }

    pub fn encode(&self, frame: &mut Frame, view: &wgpu::RawTextureView) {
        self.uniform.upload(frame);
        self.synth.encode(frame, view);
    }
}
