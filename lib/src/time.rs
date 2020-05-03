use crate::bpm_ms;
use crate::audio::Audio;

use std::collections::HashMap;

pub struct Decay {
    c: u32,
    pub t: f32,
}

impl Decay {
    // 6 place fixed-point [0.0, 1.0]
    const MAX: u32 = 1_000_000_000;

    pub fn new(t: f32) -> Self {
        Self {
            c: 0,
            t
        }
    }

    pub fn v(&self) -> f32 {
        self.c as f32 / Self::MAX as f32
    }

    pub fn update(&mut self, delta: f32) {
        let fr = delta / self.t;
        let n = (fr * Self::MAX as f32).round() as u32;

        if n > self.c {
            self.c = 0;
        } else {
            self.c -= n;
        }
    }

    pub fn set(&mut self) {
        self.c = Self::MAX;
    }

    pub fn off(&self) -> bool {
        self.c == 0
    }
}


pub struct DecayEnv {
    map: HashMap<&'static str, Decay>,
}

impl DecayEnv {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn with(mut self, key: &'static str, t: f32) -> Self {
        self.map.insert(key, Decay::new(t));
        self
    }

    pub fn v(&self, key: &'static str) -> f32 {
        self.map.get(key).unwrap().v()
    }

    pub fn update(&mut self, delta: f32) {
        self.map.values_mut().for_each(|decay| {
            decay.update(delta);
        });
    }

    pub fn set(&mut self, key: &'static str) {
        self.map.get_mut(key).unwrap().set();
    }

    pub fn off(&self, key: &'static str) -> bool {
        self.map.get(key).unwrap().off()
    }
}


pub struct BeatClock {
    pub bpm: f32,
    acc: f32,
}

impl BeatClock {
    pub fn new(bpm: f32) -> Self {
        Self {
            bpm,
            acc: 0.0,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.acc += delta;
        let ms = bpm_ms(self.bpm);

        if self.acc >= ms {
            self.acc = self.acc - ms;
            true
        } else {
            false
        }
    }
}


pub struct BeatDetect {
    pub f0: f32,
    pub f1: f32,
    pub thres: f32,
    decay: Decay,
    e0: f32,
}

impl BeatDetect {
    const BPM_MAX: f32 = 400.0;

    pub fn new(f0: f32, f1: f32, thres: f32) -> Self {
        Self {
            f0,
            f1,
            thres,
            decay: Decay::new(bpm_ms(Self::BPM_MAX)),
            e0: 0.0,
        }
    }

    pub fn update(&mut self, delta: f32, audio: &dyn Audio) -> bool {
        let (e, e0) = (audio.rms_range(self.f0, self.f1), self.e0);
        self.e0 = e;

        self.decay.update(delta);

        if e - e0 > self.thres && self.decay.off() {
            self.decay.set();
            true
        } else {
            false
        }
    }
}
