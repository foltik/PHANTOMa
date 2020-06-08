use nannou::geom::range::Range;
use nannou::math::cgmath::{Array, Matrix4, SquareMatrix, Vector3};
use nannou::prelude::TAU;
use nannou::wgpu;

use lib::gfx::{
    camera::{CameraDesc, CameraRotation},
    lights::PointLight,
    model::Model,
    scene::Scene,
};

pub struct Pillars {
    pub scene: Scene,
    pub door: f32,
    pub zoom: f32,
    pub brightness: f32,
    dz: f32,
}

// ,

impl Pillars {
    pub fn new(
        device: &wgpu::Device,
        window: &nannou::window::Window,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let pillars = Model::new(device, window, encoder, "pillars.obj");
        let scim0 = Model::new(device, window, encoder, "scim.obj");
        let scim1 = Model::new(device, window, encoder, "scim.obj");

        let scene = Scene::new(
            device,
            encoder,
            vec![pillars, scim0, scim1],
            CameraDesc {
                pos: Vector3::new(0.0, 2.0, 6.0),
                rotation: CameraRotation::LookAt(Vector3::new(0.0, 2.0, 100.0)),
                fov: 90.0,
            },
            0.0,
            vec![PointLight {
                pos: Vector3::new(0.0, 4.0, 33.0),
                ambient: Vector3::new(0.0, 0.0, 0.0),
                diffuse: Vector3::new(1.0, 0.0, 0.0),
                specular: Vector3::new(0.0, 0.0, 0.0),
                attenuation: Vector3::new(0.017, 0.07, 1.0),
            }],
        );

        Self {
            scene,
            door: 0.0,
            brightness: 0.0,
            zoom: 0.0,
            dz: 0.0,
        }
    }

    pub fn update(&mut self, index: usize, tc: f32, t: f32) {
        let light = &mut self.scene.lights.points[0];

        let ambient = Range::new(0.1, 0.0).lerp(self.brightness);
        light.ambient = Vector3::new(ambient, ambient, ambient);

        light.attenuation.x = Range::new(0.01, 0.0).lerp(self.brightness);
        light.attenuation.z = Range::new(3.0, 0.625).lerp(self.brightness);

        let model = &mut self.scene.models[0];
        model.objects[3]
            .transform
            .translate(Vector3::new(0.0, self.door * 6.01, 0.0));
        model.objects[2]
            .transform
            .translate(Vector3::new(0.0, self.door * 6.01, 0.0));

        let dy = ((t + TAU / 2.0).cos() * 0.5 + 0.5) * -0.3;
        let dz = 6.0 + 19.0 * ((((t * 2.0) + 30.0) % 60.0 - 30.0).abs() / 30.0);

        let mut mirror = Matrix4::identity();
        mirror[0][0] = -1.0f32;
        let ambient = Vector3::new(0.6, 0.9, 0.9);

        let scim0 = &mut self.scene.models[1];
        scim0.objects[0].material.desc.ambient.col = ambient;
        scim0.transform.matrix =
            Matrix4::from_translation(Vector3::new(-1.3, dy, 3.5 + dz)) * mirror;

        let scim1 = &mut self.scene.models[2];
        scim1.objects[0].material.desc.ambient.col = ambient;
        scim1.transform.translate(Vector3::new(1.3, dy, 3.5 + dz));

        let cam = &mut self.scene.camera.desc;
        if index == 0 {
            self.dz = dz;
            cam.pos.z = dz;
        } else {
            cam.pos.z += 0.4;
            cam.rotation = CameraRotation::LookAt(cam.pos + Vector3::unit_z());
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
