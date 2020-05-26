use nannou::geom::range::Range;
use nannou::math::cgmath::{Matrix4, SquareMatrix, Vector3};
use nannou::wgpu;

use lib::gfx::{camera::CameraDesc, lights::PointLight, model::Model, scene::Scene};

pub struct Pillars {
    pub scene: Scene,
    pub door: f32,
    pub zoom: f32,
    pub brightness: f32,
}

// ,

impl Pillars {
    pub fn new(
        device: &wgpu::Device,
        window: &nannou::window::Window,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let model = Model::new(device, window, encoder, "pillars.obj");

        let scene = Scene::new(
            device,
            encoder,
            vec![model],
            CameraDesc {
                eye: (0.0, 2.0, 6.0).into(),
                target: (0.0, 2.0, 40.0).into(),
                up: -Vector3::unit_y(),
                fov: 90.0,
                near: 0.1,
                far: 100.0,
            },
            0.0,
            vec![PointLight {
                pos: Vector3::new(0.0, 4.0, 33.5),
                ambient: Vector3::new(0.0, 0.0, 0.0),
                diffuse: Vector3::new(1.0, 0.0, 0.0),
                specular: Vector3::new(0.0, 0.0, 0.0),
                attenuation: Vector3::new(0.55, 0.0, 1.0),
            }],
        );

        Self {
            scene,
            door: 0.0,
            brightness: 0.0,
            zoom: 0.0,
        }
    }

    pub fn update(&mut self) {
        let light = &mut self.scene.lights.points[0];

        let ambient = Range::new(0.1, 0.0).lerp(self.brightness);
        light.ambient = Vector3::new(ambient, ambient, ambient);

        light.attenuation.x = Range::new(0.01, 0.0).lerp(self.brightness);
        light.attenuation.z = Range::new(3.0, 0.625).lerp(self.brightness);

        let cam = &mut self.scene.camera.desc;
        cam.eye.z = 6.0 + 30.0 * self.zoom;

        let door = &mut self.scene.models[0].objects[3];
        door.transform
            .translate(Vector3::new(0.0, self.door * 6.01, 0.0));
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
