use nannou::math::cgmath::{Vector3, Matrix4, Rad};
use nannou::wgpu;

use lib::gfx::{
    camera::{CameraDesc, CameraRotation},
    lights::PointLight,
    model::Model,
    scene::Scene,
};

pub struct Cube {
    pub scene: Scene,
    pub brightness: f32,
    pub red: f32,
}

impl Cube {
    pub fn new(
        device: &wgpu::Device,
        window: &nannou::window::Window,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let model = Model::new(device, window, encoder, "cube.obj");

        let scene = Scene::new(
            device,
            encoder,
            vec![model],
            CameraDesc {
                pos: Vector3::new(0.0, 0.0, -8.0),
                rotation: CameraRotation::LookAt(Vector3::new(0.0, 0.0, 0.0)),
                fov: 90.0,
            },
            0.0,
            vec![
                PointLight {
                    pos: Vector3::new(0.0, 0.0, -8.0),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(1.0, 1.0, 1.0),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(0.017, 0.07, 1.0),
                },
            ],
        );

        Self { scene, brightness: 0.0, red: 0.0 }
    }

    pub fn update(&mut self, t: f32) {
        self.scene.models[0].objects[0].transform.matrix = Matrix4::from_angle_x(Rad(t * 1.1))
        * Matrix4::from_angle_y(Rad(t * 0.8))
        * Matrix4::from_angle_z(Rad(t * 1.2));
    }

    pub fn draw(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::TextureView,
    ) {
        self.scene.update(device, encoder);
        self.scene.encode(encoder, texture);
    }
}
