use super::wgpu;
use super::uniform::{Uniform, UniformArray};
use crate::math::{Vector3, Matrix4};

pub enum LightType {
    Directional = 0,
    Point = 1,
    Spot = 2
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LightDesc {
    pub ty: u32,
    pub intensity: f32,
    pub range: f32,
    pub angle: f32,
    pub color: Vector3,
    pub pad: f32,
}

pub type LightUniform = LightDesc;

pub struct Lights {
    pub group: wgpu::BindGroup,

    pub lights: UniformArray<LightUniform>,
    pub transforms: UniformArray<Matrix4>,
    pub n: Uniform<u32>,
}

impl Lights {
    pub const MAX: usize = 32;

    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, descs: &[LightDesc], transforms: &[Matrix4]) -> Self {
        let lights = UniformArray::new(device, "lights", Self::MAX, Some(descs));
        let transforms = UniformArray::new(device, "light_transforms", Self::MAX, Some(transforms));

        let n = Uniform::new(device, "lights_n", Some(&(descs.len() as u32)));

        let group = wgpu::util::BindGroupBuilder::new("lights")
            .uniform_array(&lights)
            .uniform_array(&transforms)
            .uniform(&n)
            .build(device, layout);

        Self {
            group,
            
            lights,
            transforms,
            n
        }
    }

    pub fn bind<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, group_idx: u32) {
        pass.set_bind_group(group_idx, &self.group, &[]);
    }

    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        wgpu::util::BindGroupLayoutBuilder::new("lights")
            .uniform_array(wgpu::ShaderStages::FRAGMENT)
            .uniform_array(wgpu::ShaderStages::FRAGMENT)
            .uniform(wgpu::ShaderStages::FRAGMENT)
            .build(device)
    }
}
