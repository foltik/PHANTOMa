use crate::audio::Audio;

use std::collections::HashMap;

pub struct Decay {
    c: u32,
    hold: bool,
    pub t: f32,
}

impl Decay {
    // 6 place fixed-point [0.0, 1.0]
    const MAX: u32 = 1_000_000_000;

    pub fn new(t: f32) -> Self {
        Self { c: 0, hold: false, t }
    }

    pub fn v(&self) -> f32 {
        self.c as f32 / Self::MAX as f32
    }

    pub fn update(&mut self, delta: f32) {
        if !self.hold {
            let fr = (delta * 1000.0) / self.t;
            let n = (fr * Self::MAX as f32).round() as u32;

            if n > self.c {
                self.c = 0;
            } else {
                self.c -= n;
            }
        }
    }

    pub fn set(&mut self) {
        self.c = Self::MAX;
    }

    pub fn set_fr(&mut self, fr: f32) {
        self.c = (fr * Self::MAX as f32) as u32;
    }

    pub fn off(&self) -> bool {
        self.c == 0
    }

    pub fn hold(&mut self, hold: bool) {
        self.hold = hold;
    }
}

#[derive(Default)]
pub struct DecayEnv {
    map: HashMap<String, Decay>,
}

impl DecayEnv {
    pub fn with(mut self, key: &str, t: f32) -> Self {
        self.map.insert(key.to_owned(), Decay::new(t));
        self
    }

    pub fn get(&self, key: &str) -> &Decay {
        self.map.get(key).unwrap_or_else(|| panic!("no such decay {}", key))
    }
    pub fn get_mut(&mut self, key: &str) -> &mut Decay {
        self.map.get_mut(key).unwrap_or_else(|| panic!("no such decay {}", key))
    }

    pub fn v(&self, key: &str) -> f32 {
        self.get(key).v()
    }

    pub fn t(&mut self, key: &str, t: f32) {
        self.get_mut(key).t = t;
    }

    pub fn update(&mut self, delta: f32) {
        self.map.values_mut().for_each(|decay| {
            decay.update(delta);
        });
    }

    pub fn set(&mut self, key: &str) {
        self.get_mut(key).set();
    }

    pub fn set_fr(&mut self, key: &str, fr: f32) {
        self.get_mut(key).set_fr(fr);
    }

    pub fn off(&self, key: &str) -> bool {
        self.get(key).off()
    }

    pub fn hold(&mut self, key: &str, hold: bool) {
        self.get_mut(key).hold(hold);
    }
}

// A value smoother with a restoring force that follows Hooke's Law
pub struct Spring {
    value: f32,
    target: f32,
    k: f32,
}

impl Spring {
    // t: time to go from 0 to 1
    pub fn new(t: f32) -> Self {
        Self { 
            value: 0.0,
            target: 0.0,
            k: t,
        }
    }

    pub fn v(&self) -> f32 {
        self.value
    }

    pub fn set(&mut self, target: f32) {
        self.target = f32::max(0.0, f32::min(1.0, target));
    }

    pub fn update(&mut self, delta: f32) {
        let x = self.target - self.value;
        if x != 0.0 {
            let t = self.k * x;
            let dv = ((delta * 1000.0) / t).abs();

            self.value = f32::max(0.0, f32::min(1.0, self.value + dv * x));
        }
    }
}

pub struct BeatClock {
    pub bpm: f32,
    pub mul: f32,
    acc: f32,
}

impl BeatClock {
    pub fn new(bpm: f32) -> Self {
        Self {
            bpm,
            mul: 1.0,
            acc: 0.0,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.acc += delta * 1000.0;
        let ms = convert::bpm_ms(self.bpm) * self.mul;

        if self.acc >= ms {
            self.acc -= ms;
            true
        } else {
            false
        }
    }

    pub fn sync(&mut self) {
        self.acc = 0.0;
    }
}

pub struct BeatDetect {
    pub f0: f32,
    pub f1: f32,
    pub thres: f32,
    pub bpm_max: f32,
    decay: Decay,
    e0: f32,
}

impl BeatDetect {
    pub fn new(f0: f32, f1: f32, thres: f32, bpm_max: f32) -> Self {
        Self {
            f0,
            f1,
            thres,
            bpm_max,
            decay: Decay::new(1.0),
            e0: 0.0,
        }
    }

    pub fn update(&mut self, delta: f32, audio: &Audio) -> bool {
        let (e, e0) = (audio.rms_range(self.f0, self.f1), self.e0);
        self.e0 = e;

        self.decay.update((delta * 1000.0) / convert::bpm_ms(self.bpm_max));

        if e - e0 > self.thres && self.decay.off() {
            self.decay.set();
            true
        } else {
            false
        }
    }
}

pub mod convert {
    pub fn bpm_ms(bpm: f32) -> f32 {
        (1.0 / bpm) * 60.0 * 1000.0
    }

    pub fn ms_bpm(ms: f32) -> f32 {
        1.0 / (ms / 1000.0 / 60.0)
    }

    // pub fn fps_ms(fps: f32) -> f32 {
    // }

    // pub fn ms_fps(ms: f32) -> f32 {
    // }
}