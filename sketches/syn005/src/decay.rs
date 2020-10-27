use lib::time::Decay as DecayInstance;

pub struct Decay {
    beat: DecayInstance,
}

impl Decay {
    pub fn beat_set(&mut self) {
        self.beat.set();
    }

    pub fn beat(&self) -> f32 {
        self.beat.v()
    }

    pub fn update(&mut self, dt: f32) {
        self.beat.update(dt);
    }
}

impl Default for Decay {
    fn default() -> Self {
        Self {
            beat: DecayInstance::new(250.0)
        }
    }
}