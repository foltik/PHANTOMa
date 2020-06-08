use nannou::math::cgmath::{Matrix4, Rad, Vector3};
use nannou::geom::Range;
use nannou::wgpu;

use lib::gfx::{
    camera::{CameraDesc, CameraRotation},
    lights::PointLight,
    model::Model,
    model::Object,
    scene::Scene,
};
use lib::interp::CameraPath3D;

pub struct Church {
    pub scene: Scene,
    paths: Vec<CameraPath3D>,
    start: Vec<Option<(f32, f32)>>,
    pub offset: f32,
    pub beat: bool,
    pub brightness: f32,
}

impl Church {
    pub fn new(
        device: &wgpu::Device,
        window: &nannou::window::Window,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let model = Model::new(device, window, encoder, "church.obj");

        let paths = vec![
            CameraPath3D::new("church0.ckf", 30.0),
            CameraPath3D::new("church1.ckf", 30.0),
            CameraPath3D::new("church2.ckf", 40.0),
            CameraPath3D::new("church3.ckf", 15.0),
            CameraPath3D::new("church4.ckf", 40.0),
            CameraPath3D::new("church5.ckf", 40.0),
        ];
        let start = paths.iter().map(|_| None).collect();

        let scene = Scene::new(
            device,
            encoder,
            vec![model],
            CameraDesc {
                // pos: (13.5, 0.75, 0.0).into(),
                // rotation: CameraRotation::LookAt(Vector3::new(0.0, 0.75, 0.0)),
                pos: Vector3::new(4.227, 0.75, 10.3),
                rotation: CameraRotation::LookAt(Vector3::new(1.767, 1.514, 12.539)),
                fov: 90.0,
            },
            0.0,
            vec![
                PointLight {
                    // Top
                    pos: Vector3::new(-6.0, 11.6, 0.0),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(1.0, 0.8941, 0.7333),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(100.0, 0.14, 1.0),
                    // attenuation: Vector3::new(0.07, 0.14, 1.0),
                },
                PointLight {
                    // Cam
                    pos: Vector3::new(21.9231, 0.74536, 12.58957),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(1.0, 0.8941, 0.7333),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(100.0, 0.14, 1.0),
                    // attenuation: Vector3::new(0.017, 0.07, 1.0),
                },
                PointLight {
                    // Skull left
                    pos: Vector3::new(-12.0, 1.883, -1.864),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(0.786, 0.098, 0.048),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(1.8, 0.7, 1.0),
                },
                PointLight {
                    // Skull right
                    pos: Vector3::new(-11.881, 1.883, 2.185),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(0.786, 0.098, 0.048),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(1.8, 0.7, 1.0),
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
                PointLight {
                    // Spawn
                    pos: Vector3::new(19.812, 0.78, 10.268),
                    ambient: Vector3::new(0.0, 0.0, 0.0),
                    diffuse: Vector3::new(0.786, 0.098, 0.048),
                    specular: Vector3::new(1.0, 1.0, 1.0),
                    attenuation: Vector3::new(0.44, 0.35, 1.0),
                },
            ],
        );

        Self {
            scene,
            paths,
            start,
            offset: 0.0,
            beat: false,
            brightness: 0.0,
        }
    }

    pub fn update(&mut self, index: usize, tc: f32, t: f32) {
        let index = std::cmp::min(index, 5);

        let br = self.brightness;
        let s = 0.15;
        let rot = |o: &mut Object, p: Vector3<f32>, ofs: f32, b| {
            o.transform.matrix =
                Matrix4::from_translation(Vector3::new(p.x, p.y + (tc + ofs).sin() * b - Range::new(0.0, s / 2.0).lerp(br), p.z))
                    * Matrix4::from_scale(Range::new(1.0, 1.0 + s).lerp(br))
                    * Matrix4::from_angle_y(Rad(t))
                    * Matrix4::from_translation(-p)
        };

        let crystals = &mut self.scene.models[0].objects[53..=56];
        rot(
            &mut crystals[0],
            Vector3::new(10.743, 2.346, -2.715),
            5.4,
            0.04,
        );
        rot(
            &mut crystals[1],
            Vector3::new(5.555, 2.346, -2.715),
            3.2,
            0.04,
        );
        rot(
            &mut crystals[2],
            Vector3::new(5.606, 2.346, 2.6),
            8.8,
            0.040,
        );
        rot(
            &mut crystals[3],
            Vector3::new(10.761, 2.346, 2.709),
            9.2,
            0.04,
        );

        let mut time = |i: usize| {
            self.start[i] = self.start[i].or_else(|| Some((tc, t)));
            for (j, v) in self.start.iter_mut().enumerate() {
                if i != j {
                    *v = None;
                }
            }
            let (mtc, mt) = self.start[i].unwrap();
            (tc - mtc + self.offset, t - mt + self.offset)
        };

        let (tc, t) = time(index as usize);
        self.paths[index].update_clamp(&mut self.scene.camera, t);

        for l in &mut self.scene.lights.points[7..=11] {
            l.attenuation.x = Range::new(1.8, 0.44).lerp(self.brightness);
            l.attenuation.y = Range::new(0.7, 0.35).lerp(self.brightness);
        }

        //attenuation: Vector3::new(0.017, 0.07, 1.0),
        if index != 1 {
            let light_top = &mut self.scene.lights.points[0];
            light_top.attenuation.x = Range::new(0.07, 0.017).lerp(self.brightness);
            light_top.attenuation.y = Range::new(0.14, 0.07).lerp(self.brightness);
        } else {
            for l in &mut self.scene.lights.points[4..=6] {
                l.attenuation.x = Range::new(1.8, 0.07).lerp(self.brightness);
                l.attenuation.y = Range::new(0.7, 0.14).lerp(self.brightness);
            }
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
