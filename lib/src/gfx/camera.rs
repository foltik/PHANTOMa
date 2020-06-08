use nannou::math::cgmath::{
    self, Deg, EuclideanSpace, Euler, Matrix4, Point3, Rad, Vector3, Vector4,
};
use nannou::prelude::TAU;
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
    pos: Vector4<f32>,
}

#[derive(Debug)]
pub enum CameraRotation {
    LookAt(Vector3<f32>),
    EulerAngles(Vector3<f32>),
}

#[derive(Debug)]
pub struct CameraDesc {
    pub pos: Vector3<f32>,
    pub rotation: CameraRotation,
    pub fov: f32,
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

        let view = match d.rotation {
            CameraRotation::LookAt(target) => Matrix4::look_at(
                Point3::from_vec(d.pos),
                Point3::from_vec(target),
                -Vector3::unit_y(),
            ),
            CameraRotation::EulerAngles(a) => {
                Matrix4::from(Euler::new(Rad(a.x), Rad(a.y), Rad(a.z)))
                    * Matrix4::look_at(
                        Point3::from_vec(d.pos),
                        Point3::new(d.pos.x + 0.000001, d.pos.y - 1.0, d.pos.z),
                        -Vector3::unit_y(),
                    )
            }
        };

        let proj = cgmath::perspective(Deg(d.fov / super::ASPECT), super::ASPECT, 0.1, 1000.0);

        CameraUniform { view, proj }
    }

    fn meta(&self) -> CameraMetaUniform {
        let d = &self.desc;
        CameraMetaUniform {
            pos: Vector4::new(d.pos.x, d.pos.y, d.pos.z, 0.0),
        }
    }
}
