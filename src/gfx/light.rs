use std::cell::Cell;

use super::wgpu;
use super::frame::Frame;
use super::uniform::{Uniform, UniformArray, UniformArrayStorage};
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

impl Default for LightDesc {
    fn default() -> Self {
        Self {
            ty: 0,
            intensity: 0.0,
            range: 0.0,
            angle: 0.0,
            color: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            pad: 0.0,
        }
    }
}

pub type LightUniform = LightDesc;

pub struct Lights {
    pub group: wgpu::BindGroup,

    pub lights: UniformArrayStorage<LightUniform>,
    pub transforms: UniformArray<Matrix4>,
    pub n: Uniform<u32>,

    dirty: Cell<bool>,
}

impl Lights {
    pub const MAX: usize = 32;

    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, descs: Vec<LightDesc>, transforms: &[Matrix4]) -> Self {
        let nn = descs.len();
        let n = Uniform::new(device, "lights_n", Some(&(nn as u32)));
        let lights = UniformArrayStorage::new(device, "lights", nn, Some(descs));
        let transforms = UniformArray::new(device, "light_transforms", nn, Some(transforms));

        let group = wgpu::util::BindGroupBuilder::new("lights")
            .uniform_array(&lights.uniform)
            .uniform_array(&transforms)
            .uniform(&n)
            .build(device, layout);

        Self {
            group,
            
            lights,
            transforms,
            n,

            dirty: Cell::new(false),
        }
    }

    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        wgpu::util::BindGroupLayoutBuilder::new("lights")
            .uniform_array(wgpu::ShaderStages::FRAGMENT)
            .uniform_array(wgpu::ShaderStages::FRAGMENT)
            .uniform(wgpu::ShaderStages::FRAGMENT)
            .build(device)
    }

    pub fn bind<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, group_idx: u32) {
        pass.set_bind_group(group_idx, &self.group, &[]);
    }

    pub fn upload(&self, frame: &mut Frame) {
        if self.dirty.get() {
            self.lights.upload(frame);
            self.dirty.set(false);
        }
    }
}

impl std::ops::Index<usize> for Lights {
    type Output = LightUniform;

    fn index(&self, i: usize) -> &Self::Output {
        &self.lights[i]
    }
}

impl std::ops::IndexMut<usize> for Lights {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        self.dirty.set(true);
        &mut self.lights[i]
    }
}
