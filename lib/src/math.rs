pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;

pub type Point2 = cgmath::Point2<f32>;
pub type Point3 = cgmath::Point3<f32>;

pub type Matrix4 = cgmath::Matrix4<f32>;

pub use cgmath::{Rad, Deg, Euler};
pub use cgmath::{MetricSpace, EuclideanSpace};

pub use std::f32::consts::{PI, TAU};

pub mod prelude {
    pub use super::{PI, TAU};
    pub use super::{Vector2, Vector3, Vector4, Matrix4};
}