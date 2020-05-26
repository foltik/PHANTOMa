use nannou::math::cgmath::{self, Deg, Matrix4, Point3, Vector3, Vector4};
use nannou::wgpu;

use super::Uniform;

// TODO: Have a Uniform trait that we can derive to
// auto generate thing that returns self.clone
//, OR we can overload it (with CameraDesc -> CameraUniform)
// to transform data before upload

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CameraUniform {
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CameraMetaUniform {
    eye: Vector4<f32>,
}

#[derive(Debug)]
pub struct CameraDesc {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

pub struct Camera {
    pub desc: CameraDesc,
    pub transform: Uniform<CameraUniform>,
    pub meta: Uniform<CameraMetaUniform>,
}

impl Camera {
    pub fn new(device: &wgpu::Device, desc: CameraDesc) -> Self {
        Self {
            desc,
            transform: Uniform::new(device),
            meta: Uniform::new(device),
        }
    }

    pub fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        self.transform.upload(device, encoder, self.transform());
        self.meta.upload(device, encoder, self.meta());
    }

    fn transform(&self) -> CameraUniform {
        let d = &self.desc;
        let view = Matrix4::look_at(d.eye, d.target, d.up);
        let proj = cgmath::perspective(Deg(d.fov / super::ASPECT), super::ASPECT, d.near, d.far);

        CameraUniform { view, proj }
    }

    fn meta(&self) -> CameraMetaUniform {
        let d = &self.desc;
        CameraMetaUniform {
            eye: Vector4::new(d.eye.x, d.eye.y, d.eye.z, 0.0),
        }
    }
}
