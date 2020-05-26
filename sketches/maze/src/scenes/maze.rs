use nannou::math::cgmath::{EuclideanSpace, Point3, Vector2, Vector3};
use nannou::wgpu;

use lib::{
    gfx::{camera::CameraDesc, lights::PointLight, model::Model, scene::Scene},
    interp::{self, Spline},
};

pub struct Maze {
    pub scene: Scene,
    pub path: Spline<f32, Vector2<f32>>,
}

impl Maze {
    pub fn new(
        device: &wgpu::Device,
        window: &nannou::window::Window,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let model = Model::new(device, window, encoder, "maze.obj");

        let scene = Scene::new(
            device,
            encoder,
            vec![model],
            CameraDesc {
                eye: (0.0, 0.75, 0.0).into(),
                target: (0.0, 0.75, 1.0).into(),
                up: -Vector3::unit_y(),
                fov: 90.0,
                near: 0.1,
                far: 100.0,
            },
            0.0,
            vec![PointLight {
                pos: Vector3::new(0.0, 0.75, 0.0),
                ambient: Vector3::new(0.0, 0.0, 0.0),
                diffuse: Vector3::new(1.0, 1.0, 1.0),
                specular: Vector3::new(0.0, 0.0, 0.0),
                attenuation: Vector3::new(0.1, 0.7, 1.0),
            }],
        );

        let path = interp::catmull_loop(
            &vec![
                Vector2::new(0.0, 0.0),
                Vector2::new(0.0, 6.0),
                Vector2::new(8.0, 6.0),
                Vector2::new(8.0, 10.0),
                Vector2::new(14.0, 10.0),
                Vector2::new(14.0, 16.0),
                Vector2::new(0.0, 16.0),
                Vector2::new(0.0, 6.0),
                Vector2::new(8.0, 6.0),
                Vector2::new(8.0, 10.0),
                Vector2::new(18.0, 10.0),
                Vector2::new(18.0, 0.0),
                Vector2::new(12.0, 0.0),
                Vector2::new(12.0, -6.0),
                Vector2::new(0.0, -6.0),
            ],
            25.0,
        );

        Self { scene, path }
    }

    pub fn update(&mut self, t: f32) {
        let curr = self.path.sample(t % 25.0).unwrap();
        let next = self.path.sample((t + 0.1) % 25.0).unwrap();

        let cam = &mut self.scene.camera.desc;
        cam.eye = Point3::new(curr.x, 0.75, curr.y);
        cam.target = Point3::new(next.x, 0.75, next.y);

        let light = &mut self.scene.lights.points[0];
        light.pos = cam.eye.to_vec();
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
