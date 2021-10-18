use lib::gfx::uniform::UniformStorage;
use lib::gfx::pass::SynthPass;
use lib::gfx::frame::Frame;
use lib::gfx::wgpu;

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct Stripes {
    pub color: [f32; 3],
    pub n: u32,
    pub dir: u32,
    pub duty: f32,
    pub fade_l: f32,
    pub fade_r: f32,
}

pub struct StripesPass {
    uniform: UniformStorage<Stripes>,
    synth: SynthPass,
}

impl StripesPass {
    pub fn new(device: &wgpu::Device, size: (usize, usize), stripes: Stripes) -> Self {
        let uniform = UniformStorage::new(device, "stripes", stripes);
        let synth = SynthPass::new(device, "stripes", "line_stripes.frag.spv", Some(&uniform.uniform));

        Self {
            uniform,
            synth,
        }
    }

    pub fn encode(&self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        self.uniform.upload(frame);
        self.synth.encode(frame, target);
    }
}

impl std::ops::Deref for StripesPass {
    type Target = Stripes;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl std::ops::DerefMut for StripesPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}
