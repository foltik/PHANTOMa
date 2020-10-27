use lib::app::App;

use lib::math::Vector2;

use lib::gfx::uniform::UniformStorage;
use lib::gfx::wgpu;

pub struct MaterialDesc {
    pub name: &'static str,
    pub nodes: Vec<&'static str>,
    pub images: Vec<&'static str>,
    pub fps: usize,
    pub unlit: bool,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MaterialUniform {
    pub scale: Vector2,
    pub i: u32,
    pub unlit: u32,
}

pub struct Material {
    pub name: String,
    pub fps: u32,
    pub uniform: UniformStorage<MaterialUniform>,
    group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        app: &App,
        desc: &MaterialDesc,
        scale: Vector2,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        view: &wgpu::TextureView,
    ) -> Self {
        let uniform = UniformStorage::new(
            &app.device,
            &format!("material_{}", &desc.name),
            MaterialUniform {
                scale,
                i: 0,
                unlit: if desc.unlit { 1 } else { 0 },
            },
        );

        let group = wgpu::util::BindGroupBuilder::new(&desc.name)
            .texture(&view)
            .sampler(sampler)
            .uniform(&uniform.as_ref())
            .build(&app.device, layout);

        Self {
            name: desc.name.to_owned(),
            fps: desc.fps as u32,
            uniform,
            group,
        }
    }

    pub fn bind<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, group_idx: u32) {
        pass.set_bind_group(group_idx, &self.group, &[]);
    }
}
