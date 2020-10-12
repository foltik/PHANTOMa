use crate::math::{Deg, Rad, Euler};
use crate::math::{Point3, Vector3, Matrix4};
use crate::math::EuclideanSpace;

use super::uniform::Uniform;

// TODO: Have a Uniform trait that we can derive to
// auto generate thing that returns self.clone
//, OR we can overload it (with CameraDesc -> CameraUniform)
// to transform data before upload

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CameraTransform {
    view: Matrix4,
    proj: Matrix4,
    pos: Point3,
}

impl CameraTransform {
    pub fn new(d: &CameraDesc) -> Self {
        let view = match d.rotation {
            CameraRotation::LookAt(target) => Matrix4::look_at(
                d.pos,
                Point3::from_vec(target),
                -Vector3::unit_y(),
            ),
            CameraRotation::EulerAngles(a) => {
                Matrix4::from(Euler::new(Rad(a.x), Rad(a.y), Rad(a.z)))
                    * Matrix4::look_at(
                        d.pos,
                        Point3::new(d.pos.x + 0.000001, d.pos.y - 1.0, d.pos.z),
                        -Vector3::unit_y(),
                    )
            }
        };

        let proj = cgmath::perspective(Deg(d.fov / d.aspect), d.aspect, 0.1, 1000.0);

        Self { view, proj, pos: d.pos }
    }
}

#[derive(Debug)]
pub enum CameraRotation {
    LookAt(Vector3),
    EulerAngles(Vector3),
}

#[derive(Debug)]
pub struct CameraDesc {
    pub pos: Point3,
    pub rotation: CameraRotation,
    pub fov: f32,
    pub aspect: f32,
}

pub struct Camera {
    pub desc: CameraDesc,
    pub uniform: Uniform<CameraTransform>,
}

impl Camera {
    pub fn new(device: &wgpu::Device, desc: CameraDesc) -> Self {
        let transform = &CameraTransform::new(&desc);
        Self {
            desc,
            uniform: Uniform::new(device, "camera", Some(transform)),
        }
    }

    pub fn update(&self, frame: &mut crate::gfx::frame::Frame) {
        self.uniform.upload(frame, &CameraTransform::new(&self.desc));
    }
}
