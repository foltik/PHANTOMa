use crate::math::prelude::*;

pub fn rand(seed: f32) -> f32 {
    let v = v2(seed + 10.0, seed + 3.0);
    let dt = v.perp_dot(v2(12.9898, 78.233));
    let sn = dt % PI;
    (sn.sin() * 43758.547).fract()
}

struct CharsIter {
    seed: f32,
}

impl CharsIter {
    const CHARS: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
}

impl Iterator for CharsIter {
    type Item = &'static str;

    fn next(&mut self) -> Option<Self::Item> {
        self.seed += 1000.0;
        let i = (rand(self.seed) * Self::CHARS.len() as f32).trunc() as isize;
        Some(unsafe {
            let ptr = Self::CHARS.as_ptr().offset(i);
            let slice = std::slice::from_raw_parts(ptr, 1);
            std::str::from_utf8_unchecked(slice)
        })
    }
}

pub fn chars(seed: f32) -> impl Iterator<Item = &'static str> {
    CharsIter { seed }
}
