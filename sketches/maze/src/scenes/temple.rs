use nannou::math::cgmath::{Vector2, Vector3, Array};
use nannou::geom::Range;
use nannou::prelude::{TAU, PI};
use nannou::wgpu;

use lib::gfx::{
    camera::{CameraDesc, CameraRotation},
    lights::PointLight,
    model::Model,
    scene::Scene,
};

pub struct Temple {
    pub scene: Scene,
    pub brightness: f32,
    pub red: f32,
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
                pos: Vector3::new(12.0, -1.0, 0.0),
                rotation: CameraRotation::LookAt(Vector3::new(0.0, -1.5, 0.0)),
                fov: 90.0,
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

        Self { scene, brightness: 0.0, red: 0.0 }
    }

    pub fn update(&mut self, t: f32) {
        let cam = &mut self.scene.camera.desc;

        let pd = 5.0;
        let hpd = pd / 2.0;

        let dt = 1.25 * PI;
        let ct = ((((t * 0.5)) % pd - hpd).abs() / hpd) * 0.5 * PI;
        let cr = 8.0 + 4.0 * ((t * 0.25).sin() * 0.5 + 0.5);
        let cpos = Vector2::new((ct + dt).cos(), (ct + dt).sin()) * cr;

        cam.pos = Vector3::new(cpos.x, -1.0, cpos.y);
        cam.rotation = CameraRotation::LookAt(Vector3::new(0.0, -1.0, 0.0));

        let emissive = &mut self.scene.models[0].objects[0];
        let gb = Range::new(1.0, 0.5 - self.red).lerp(self.brightness);
        let color = Vector3::new(1.0, gb, gb);
        emissive.material.desc.emissive.col = color;
        emissive.material.desc.diffuse.col = color;
        emissive.material.desc.specular.col = Vector3::from_value(0.0);

        for light in &mut self.scene.lights.points {
            light.diffuse = color;
        }
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
