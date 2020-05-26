use nannou::math::cgmath::{Vector3, Point3};
use nannou::wgpu;

use lib::gfx::{camera::CameraDesc, lights::PointLight, model::Model, scene::Scene};

pub struct Temple {
    scene: Scene,
}

impl Temple {
    pub fn new(
        device: &wgpu::Device,
        window: &nannou::window::Window,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let model = Model::new(device, window, encoder, "temple.obj");

        let scene = Scene::new(
            device,
            encoder,
            vec![model],
            CameraDesc {
                eye: (12.0, -1.0, 0.0).into(),
                target: (0.0, -1.5, 0.0).into(),
                up: -Vector3::unit_y(),
                fov: 90.0,
                near: 0.1,
                far: 100.0,
            },
            0.0,
            vec![
                PointLight {
                    pos: Vector3::new(0.0, 1.5, -1.5),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(1.0, 1.0, 1.0),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(0.15, 0.1, 1.0),
                },
                PointLight {
                    pos: Vector3::new(1.0, 0.0, -1.5),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(1.0, 1.0, 1.0),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(0.15, 0.1, 1.0),
                },
                PointLight {
                    pos: Vector3::new(-1.0, 0.0, -1.5),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(1.0, 1.0, 1.0),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(0.15, 0.1, 1.0),
                },
                PointLight {
                    pos: Vector3::new(1.0, -1.5, -0.5),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(1.0, 1.0, 1.0),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(0.15, 0.1, 1.0),
                },
                PointLight {
                    pos: Vector3::new(-1.0, -1.5, -0.5),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(1.0, 1.0, 1.0),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(0.15, 0.1, 1.0),
                },
            ],
        );

        Self { scene }
    }

    pub fn update(&mut self, t: f32) {
        let cam = &mut self.scene.camera.desc;
        cam.eye = Point3::new(t.cos() * 12.0, -1.0, t.sin() * 12.0);
        cam.target = Point3::new(0.0, -1.5, 0.0);
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
