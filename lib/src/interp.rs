use nannou::math::cgmath::MetricSpace;
use splines::{Key, Interpolation};

pub use splines::Spline;

// Compute partial sums of distances between each point
pub fn partial_distance<T: MetricSpace<Metric = f32> + Copy>(points: &[T]) -> Vec<f32> {
    let mut dists = Vec::with_capacity(points.len());
    let mut total = 0.0;
    let mut last: Option<T> = None;
    for p in points {
        let d = match last {
            Some(o) => o.distance(*p),
            None => 0.0
        };
        last = Some(*p);
        total += d;
        dists.push(total);
    }

    dists
}

pub fn catmull<T: MetricSpace<Metric = f32> + Copy>(points: &[T], t: f32) -> Spline<f32, T> {
    let n = points.len();
    assert!(n >= 3);

    // Compute normalized distances from [0, t]
    let mut dists = partial_distance(points);
    let total = dists[n - 1];
    dists.iter_mut().for_each(|d| { *d = (*d / total) * t; });

    let mut keys = Vec::new();

    keys.push(Key::new(0.0, points[0], Interpolation::default()));
    keys.extend(points[0..n - 1].iter().zip(dists.iter()).map(|(p, t)| {
        Key::new(*t, *p, Interpolation::CatmullRom)
    }));
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
    dists.iter_mut().for_each(|d| { *d = (*d / total) * t; });

    let mut keys = Vec::new();

    keys.push(Key::new(0.0, points[n - 1], Interpolation::default()));
    keys.extend(points[0..n].iter().zip(dists.iter()).map(|(p, t)| {
        Key::new(*t, *p, Interpolation::CatmullRom)
    }));
    keys.push(Key::new(t, points[0], Interpolation::CatmullRom));
    keys.push(Key::new(t + dists[1], points[1], Interpolation::default()));
    keys.push(Key::new(t + dists[2], points[2], Interpolation::default()));

    Spline::from_vec(keys)
}
