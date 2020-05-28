use nannou::math::cgmath::{Matrix4, Rad, Vector3};
use nannou::wgpu;

use lib::gfx::{camera::CameraDesc, lights::PointLight, model::Model, model::Object, scene::Scene};

pub struct Church {
    pub scene: Scene,
}

impl Church {
    pub fn new(
        device: &wgpu::Device,
        window: &nannou::window::Window,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let model = Model::new(device, window, encoder, "church.obj");

        let scene = Scene::new(
            device,
            encoder,
            vec![model],
            CameraDesc {
                // eye: (13.5, 0.75, 0.0).into(),
                // target: (0.0, 0.75, 0.0).into(),
                eye: (4.227, 0.75, 10.3).into(),
                target: (1.767, 1.514, 12.539).into(),
                up: -Vector3::unit_y(),
                fov: 90.0,
                near: 0.1,
                far: 100.0,
            },
            0.0,
            vec![
                PointLight {
                    // Top
                    pos: Vector3::new(-6.0, 11.6, 0.0),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(1.0, 0.8941, 0.7333),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(0.017, 0.07, 1.0),
                },
                PointLight {
                    // Skull left
                    pos: Vector3::new(-12.0, 1.883, -1.864),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(1.0, 1.0, 1.0),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(0.2, 0.22, 1.0),
                },
                PointLight {
                    // Skull right
                    pos: Vector3::new(-11.881, 1.883, 2.185),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(1.0, 1.0, 1.0),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(0.2, 0.22, 1.0),
                },
                PointLight {
                    // Lava bottom
                    pos: Vector3::new(1.82, 0.13, 12.8),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(0.786, 0.098, 0.048),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(0.07, 0.14, 1.0),
                },
                PointLight {
                    // Lava top left
                    pos: Vector3::new(1.459, 0.896, 12.539),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(0.786, 0.098, 0.048),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(0.07, 0.14, 1.0),
                },
                PointLight {
                    // Lava top right
                    pos: Vector3::new(1.943, 0.896, 12.539),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(0.786, 0.098, 0.048),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(0.07, 0.14, 1.0),
                },
                PointLight {
                    // Gargoyle bl
                    pos: Vector3::new(10.713, 2.149, -2.731),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(0.786, 0.098, 0.048),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(1.8, 0.7, 1.0),
                },
                PointLight {
                    // Gargoyle br
                    pos: Vector3::new(10.794, 2.149, 2.744),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(0.786, 0.098, 0.048),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(1.8, 0.7, 1.0),
                },
                PointLight {
                    // Gargoyle fr
                    pos: Vector3::new(5.644, 2.149, 2.744),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(0.786, 0.098, 0.048),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(1.8, 0.7, 1.0),
                },
                PointLight {
                    // Gargoyle fl
                    pos: Vector3::new(5.561, 2.149, -2.711),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(0.786, 0.098, 0.048),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(1.8, 0.7, 1.0),
                },
            ],
        );

        Self { scene }
    }

    pub fn update(&mut self, tc: f32, t: f32) {
        let rot = |o: &mut Object, p: Vector3<f32>, b| {
            o.transform.matrix =
                Matrix4::from_translation(Vector3::new(p.x, p.y + tc.sin() * b, p.z))
                    * Matrix4::from_angle_y(Rad(t))
                    * Matrix4::from_translation(-p)
        };

        let crystals = &mut self.scene.models[0].objects[53..=56];
        rot(&mut crystals[0], Vector3::new(10.743, 2.346, -2.715), 0.025);
        rot(&mut crystals[1], Vector3::new(5.555, 2.346, -2.715), 0.025);
        rot(&mut crystals[2], Vector3::new(5.606, 2.346, 2.6), 0.025);
        rot(&mut crystals[3], Vector3::new(10.761, 2.346, 2.709), 0.025);
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
