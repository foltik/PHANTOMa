use super::frame::Frame;
use super::uniform::{Uniform, UniformArray};
use super::wgpu;
use crate::math::Vector3;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub intensity: f32,
    pub falloff: f32,
    pub range: f32,
    pub color: Vector3,
}

pub struct Lights {
    pub points: Vec<PointLight>,
    pub uniform: UniformArray<PointLight>,
    pub n: Uniform<u32>,
}

impl Lights {
    pub const MAX_POINT: usize = 16;

    pub fn new(device: &wgpu::Device, points: Vec<PointLight>) -> Self {
        let n = points.len() as u32;
        Self {
            points,
            uniform: UniformArray::new(device, "lights", Self::MAX_POINT),
            n: Uniform::new(device, "lights_n", Some(&n)),
        }
    }

    pub fn update_one(&self, frame: &mut Frame, i: usize, light: &PointLight) {
        self.uniform.upload_el(frame, i, light);
    }

    pub fn update(&self, frame: &mut Frame) {
        self.uniform.upload(frame, &self.points);
    }
}
