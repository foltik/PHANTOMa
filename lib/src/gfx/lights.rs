use nannou::math::cgmath::{Vector3, Vector4};
use nannou::wgpu;

use super::Uniform;

pub struct PointLight {
    pos: Vector3<f32>,
    ambient: Vector3<f32>,
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
    attenuation: Vector3<f32>,
}

impl PointLight {
    fn uniform(&self) -> PointLightUniform {
        PointLightUniform {
            pos: Vector4::new(self.pos.x, self.pos.y, self.pos.z, 0.0),
            ambient: Vector4::new(self.ambient.x, self.ambient.y, self.ambient.z, 0.0),
            diffuse: Vector4::new(self.diffuse.x, self.diffuse.y, self.diffuse.z, 0.0),
            specular: Vector4::new(self.specular.x, self.specular.y, self.specular.z, 0.0),
            attenuation: Vector4::new(
                self.attenuation.x,
                self.attenuation.y,
                self.attenuation.z,
                0.0,
            ),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PointLightUniform {
    pos: Vector4<f32>,
    ambient: Vector4<f32>,
    diffuse: Vector4<f32>,
    specular: Vector4<f32>,
    attenuation: Vector4<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LightsInfoUniform {
    n: Vector4<f32>,
}

pub struct Lights {
    pub points: Vec<PointLight>,
    pub points_buffer: wgpu::Buffer,
    pub info: Uniform<LightsInfoUniform>,
}

impl Lights {
    pub const MAX_POINT: usize = 16;

    pub fn new(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        points: Vec<PointLight>,
    ) -> Self {
        let points_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: (Self::MAX_POINT * std::mem::size_of::<PointLightUniform>())
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let lights = Self {
            points,
            points_buffer,
            info: Uniform::new(device),
        };

        lights.update(device, encoder);

        lights
    }

    pub fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        let n_points = self.points.len();
        assert!(
            n_points < Self::MAX_POINT,
            "{} > {}",
            n_points,
            Self::MAX_POINT
        );

        let points_uniform = self.points.iter().map(|p| p.uniform()).collect::<Vec<_>>();

        let points_staging = device
            .create_buffer_mapped(n_points, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&points_uniform);

        encoder.copy_buffer_to_buffer(
            &points_staging,
            0,
            &self.points_buffer,
            0,
            (n_points * std::mem::size_of::<PointLightUniform>()) as wgpu::BufferAddress,
        );

        let info = LightsInfoUniform {
            n: Vector4::new(n_points as f32, 0.0, 0.0, 0.0),
        };

        self.info.upload(device, encoder, info);
    }
}
