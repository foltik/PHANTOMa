use image::DynamicImage;

use crate::math::{Vector3, Vector4};

pub enum MaterialAttr<V> {
    Value(V),
    Map(DynamicImage),
}

pub struct MaterialDesc {
    pub name: String,
    pub color: MaterialAttr<Vector4>,
    pub emissive: Vector3,
    pub unlit: bool,
}