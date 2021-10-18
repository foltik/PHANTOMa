use lib::gfx::uniform::UniformStorage;
use lib::gfx::pass::SynthPass;
use lib::gfx::frame::Frame;
use lib::gfx::wgpu;

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct Tiles {
    pub color: [f32; 3],
    pub nx: f32,
    pub ny: f32,
    pub t: f32,
}

pub struct TilesPass {
    uniform: UniformStorage<Tiles>,
    synth: SynthPass,
}

impl TilesPass {
    pub fn new(device: &wgpu::Device, size: (usize, usize), tiles: Tiles) -> Self {
        let uniform = UniformStorage::new(device, "tiles", tiles);
        let synth = SynthPass::new(device, "tiles", "tiles.frag.spv", Some(&uniform.uniform));

        Self {
            uniform,
            synth,
        }
    }

    pub fn update(&mut self, t: f32) {
        self.uniform.t = t;
    }

    pub fn encode(&self, frame: &mut Frame, target: &wgpu::RawTextureView) {
        self.uniform.upload(frame);
        self.synth.encode(frame, target);
    }
}

impl std::ops::Deref for TilesPass {
    type Target = Tiles;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl std::ops::DerefMut for TilesPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}
