use nannou::math::cgmath::{MetricSpace, Vector3};
use nannou::prelude::TAU;
use splines::{Interpolation, Key};

pub use splines::Spline;

use crate as lib;
use crate::gfx::camera::{Camera, CameraRotation};

// Compute partial sums of distances between each point
pub fn partial_distance<T: MetricSpace<Metric = f32> + Copy>(points: &[T]) -> Vec<f32> {
    let mut dists = Vec::with_capacity(points.len());
    let mut total = 0.0;
    let mut last: Option<T> = None;
    for p in points {
        let d = match last {
            Some(o) => o.distance(*p),
            None => 0.0,
        };
        last = Some(*p);
        total += d;
        dists.push(total);
    }

    dists
}

pub fn linear<T: MetricSpace<Metric = f32> + Copy>(points: &[T], t: f32) -> Spline<f32, T> {
    let n = points.len();
    assert!(n >= 3);

    // Compute normalized distances from [0, t]
    let mut dists = partial_distance(points);
    let total = dists[n - 1];
    dists.iter_mut().for_each(|d| {
        *d = (*d / total) * t;
    });

    let mut keys = Vec::new();

    keys.extend(
        points[0..n - 1]
            .iter()
            .zip(dists.iter())
            .map(|(p, t)| Key::new(*t, *p, Interpolation::Linear)),
    );
    keys.push(Key::new(t + 0.1, points[n - 1], Interpolation::Linear));

    Spline::from_vec(keys)
}

pub fn catmull<T: MetricSpace<Metric = f32> + Copy>(points: &[T], t: f32) -> Spline<f32, T> {
    let n = points.len();
    assert!(n >= 3);

    // Compute normalized distances from [0, t]
    let mut dists = partial_distance(points);
    let total = dists[n - 1];
    dists.iter_mut().for_each(|d| {
        *d = (*d / total) * t;
    });

    let mut keys = Vec::new();

    keys.push(Key::new(0.0, points[0], Interpolation::default()));
    keys.extend(
        points[0..n - 1]
            .iter()
            .zip(dists.iter())
            .map(|(p, t)| Key::new(*t, *p, Interpolation::CatmullRom)),
    );
    keys.push(Key::new(t, points[n - 1], Interpolation::default()));
    keys.push(Key::new(t, points[n - 1], Interpolation::default()));

    Spline::from_vec(keys)
}

pub fn catmull_loop<T: MetricSpace<Metric = f32> + Copy>(points: &[T], t: f32) -> Spline<f32, T> {
    let n = points.len();
    assert!(n >= 3);

    // Compute normalized distances from [0, t] plus the connecting distance
    let mut dists = partial_distance(points);
    let join = points[n - 1].distance(points[0]);
    let total = dists[n - 1] + join;
    dists.push(total);
    dists.iter_mut().for_each(|d| {
        *d = (*d / total) * t;
    });

    let mut keys = Vec::new();

    keys.push(Key::new(0.0, points[n - 1], Interpolation::default()));
    keys.extend(
        points[0..n]
            .iter()
            .zip(dists.iter())
            .map(|(p, t)| Key::new(*t, *p, Interpolation::CatmullRom)),
    );
    keys.push(Key::new(t, points[0], Interpolation::CatmullRom));
    keys.push(Key::new(t + dists[1], points[1], Interpolation::default()));
    keys.push(Key::new(t + dists[2], points[2], Interpolation::default()));

    Spline::from_vec(keys)
}

pub struct CameraPath3D {
    position: Spline<f32, Vector3<f32>>,
    rotation: Spline<f32, Vector3<f32>>,
    t: f32,
}

impl CameraPath3D {
    pub fn new(file: &str, t: f32) -> Self {
        let data = lib::read_resource(file);

        let mut keyframes = data
            .split("\n")
            .map(|line| line.split(",").map(|f| f.parse::<f32>().unwrap()));

        let rx = keyframes.next().unwrap();
        let ry = keyframes.next().unwrap();
        let rz = keyframes.next().unwrap();
        let angles = rx
            .zip(ry)
            .zip(rz)
            .map(|((x, y), z)| Vector3::new(x, y, z + TAU / 4.0))
            .collect::<Vec<_>>();

        let px = keyframes.next().unwrap();
        let py = keyframes.next().unwrap();
        let pz = keyframes.next().unwrap();
        let points = px
            .zip(py)
            .zip(pz)
            .map(|((x, y), z)| Vector3::new(x, z, y))
            .collect::<Vec<_>>();

        let position = linear(&points, t);
        let rotation = linear(&angles, t);

        Self {
            position,
            rotation,
            t,
        }
    }

    fn update_internal(&self, camera: &mut Camera, t: f32) {
        let pos = self.position.sample(t).unwrap();
        let rot = self.rotation.sample(t).unwrap();

        camera.desc.pos = pos;
        camera.desc.rotation = CameraRotation::EulerAngles(rot);
    }

    pub fn update(&self, camera: &mut Camera, t: f32) {
        self.update_internal(camera, t % self.t);
    }

    pub fn update_clamp(&self, camera: &mut Camera, t: f32) {
        self.update_internal(camera, t.min(self.t));
    }
}
