use crate::math::Matrix4;

use super::wgpu;
use super::uniform::Uniform;

#[derive(Debug)]
pub struct Camera {
    pub group: wgpu::BindGroup,

    pub view: Uniform<Matrix4>,
    pub proj: Uniform<Matrix4>,
}

impl Camera {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, view: &Matrix4, proj: &Matrix4) -> Self {
        let view = Uniform::new(device, "cam_view", Some(view));
        let proj = Uniform::new(device, "cam_proj", Some(proj));

        let group = wgpu::util::BindGroupBuilder::new("cam")
            .uniform(&view)
            .uniform(&proj)
            .build(device, layout);

        Self {
            group,

            view,
            proj,
        }
    }

    pub fn bind<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, group_idx: u32) {
        pass.set_bind_group(group_idx, &self.group, &[]);
    }

    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        wgpu::util::BindGroupLayoutBuilder::new("cam")
            .uniform(wgpu::ShaderStages::VERTEX)
            .uniform(wgpu::ShaderStages::VERTEX)
            .build(device)
    }
}