use nannou::math::cgmath::{Vector3, Vector4};
use nannou::wgpu;

use super::Uniform;

pub struct PointLight {
    pub pos: Vector3<f32>,
    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,
    pub attenuation: Vector3<f32>,
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
    info: Vector4<f32>,
}

pub struct Lights {
    pub ambient: f32,
    pub points: Vec<PointLight>,
    pub points_uniform: Uniform<PointLightUniform>,
    pub info_uniform: Uniform<LightsInfoUniform>,
}

impl Lights {
    pub const MAX_POINT: usize = 16;

    pub fn new(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        ambient: f32,
        points: Vec<PointLight>,
    ) -> Self {
        let lights = Self {
            ambient,
            points,
            points_uniform: Uniform::new_array(device, Self::MAX_POINT),
            info_uniform: Uniform::new(device),
        };

        lights.update(device, encoder);

        lights
    }

    pub fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        let n_points = self.points.len();
        assert!(n_points < Self::MAX_POINT, "max point lights exceeded");

        if n_points > 0 {
            let uniforms = self.points.iter().map(|p| p.uniform()).collect::<Vec<_>>();
            self.points_uniform.upload_slice(device, encoder, &uniforms);
        }

        self.info_uniform.upload(
            device,
            encoder,
            LightsInfoUniform {
                info: Vector4::new(self.ambient, n_points as f32, 0.0, 0.0),
            },
        );
    }
}
