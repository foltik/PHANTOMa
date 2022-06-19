pub use cgmath::SquareMatrix;
pub use cgmath::Zero;
pub use cgmath::{Deg, Rad, Euler};
pub use cgmath::{EuclideanSpace, InnerSpace, MetricSpace, VectorSpace};

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;

pub type Point2 = cgmath::Point2<f32>;
pub type Point3 = cgmath::Point3<f32>;

// pub type Euler = cgmath::Euler<f32>;
pub type Quat = cgmath::Quaternion<f32>;

pub type Matrix4 = cgmath::Matrix4<f32>;

pub use std::f32::consts::{PI, TAU};

pub fn p2(x: f32, y: f32) -> Point2 {
    Point2::new(x, y)
}

pub fn p3(x: f32, y: f32, z: f32) -> Point3 {
    Point3::new(x, y, z)
}

pub fn v2(x: f32, y: f32) -> Vector2 {
    Vector2::new(x, y)
}

pub fn v3(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3::new(x, y, z)
}

pub fn v4(x: f32, y: f32, z: f32, w: f32) -> Vector4 {
    Vector4::new(x, y, z, w)
}

pub fn q3(x: f32, y: f32, z: f32) -> Quat {
    Euler {
        x: Rad(x),
        y: Rad(y),
        z: Rad(z)
    }.into()
}

pub mod prelude {
    pub use super::SquareMatrix;
    pub use super::Zero;
    pub use super::{p2, p3, v2, v3, v4, q3};
    pub use super::{Deg, Rad, Euler};
    pub use super::{EuclideanSpace, InnerSpace, MetricSpace, VectorSpace};
    pub use super::{Matrix4, Quat, Vector2, Vector3, Vector4};
    pub use super::{PI, TAU};
    pub use min_max::{min, max, min_partial, max_partial, min_max, min_max_partial};
}

pub mod projection {
    pub use cgmath::{frustum, ortho, perspective};
}
