use nannou::math::cgmath::{Vector3 as CVector3, Vector4 as CVector4};
use nannou::wgpu;

type Vector3 = CVector3<f32>;
type Vector4 = CVector4<f32>;

use super::Uniform;
use crate as lib;

pub struct MaterialAttrDesc {
    pub col: Vector3,
    pub map: Option<String>,
}

pub struct MaterialDesc {
    pub ambient: MaterialAttrDesc,
    pub diffuse: MaterialAttrDesc,
    pub specular: MaterialAttrDesc,
    pub emissive: MaterialAttrDesc,
    pub alpha: f32,
}

impl MaterialDesc {
    fn uniform(&self) -> MaterialUniform {
        MaterialUniform {
            ambient: Vector4::new(
                self.ambient.col.x,
                self.ambient.col.y,
                self.ambient.col.z,
                0.0,
            ),
            diffuse: Vector4::new(
                self.diffuse.col.x,
                self.diffuse.col.y,
                self.diffuse.col.z,
                0.0,
            ),
            specular: Vector4::new(
                self.specular.col.x,
                self.specular.col.y,
                self.specular.col.z,
                0.0,
            ),
            emissive: Vector4::new(
                self.emissive.col.x,
                self.emissive.col.y,
                self.emissive.col.z,
                0.0,
            ),
            extra: Vector4::new(self.alpha, 0.0, 0.0, 0.0),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MaterialUniform {
    pub ambient: Vector4,
    pub diffuse: Vector4,
    pub specular: Vector4,
    pub emissive: Vector4,
    pub extra: Vector4,
}

pub struct Material {
    pub desc: MaterialDesc,
    pub uniform: Uniform<MaterialUniform>,
    pub ambient: Option<wgpu::TextureView>,
    pub diffuse: Option<wgpu::TextureView>,
    pub specular: Option<wgpu::TextureView>,
    pub emissive: Option<wgpu::TextureView>,
    pub filter: wgpu::FilterMode,
}

impl Material {
    pub fn new(device: &wgpu::Device, window: &nannou::window::Window, desc: MaterialDesc) -> Self {
        let load_map = |map: &Option<String>| {
            map.as_ref().map(|file| {
                let image =
                    nannou::image::open(lib::resource(file)).expect(&format!("{} not found", file));
                let texture = wgpu::Texture::from_image(window, &image);
                texture.view().build()
            })
        };

        Self {
            ambient: load_map(&desc.ambient.map),
            diffuse: load_map(&desc.diffuse.map),
            specular: load_map(&desc.specular.map),
            emissive: load_map(&desc.emissive.map),
            desc,
            uniform: Uniform::new(device),
            filter: wgpu::FilterMode::Nearest,
        }
    }

    pub fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        self.uniform.upload(device, encoder, self.desc.uniform());
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.uniform.buffer
    }
}
