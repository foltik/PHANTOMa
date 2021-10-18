use lib::audio::Audio;

pub struct Time {
    t: f32,

    rms_t: f32,
    rms_scale: f32,

    mul_t: f32,
    mul_scale: f32,
}

impl Time {
    pub fn new() -> Self {
        Self {
            t: 0.0,

            rms_t: 0.0,
            rms_scale: 1.0,

            mul_t: 0.0,
            mul_scale: 1.0,
        }
    }

    pub fn update(&mut self, dt: f32, audio: Option<&Audio>) {
        self.t += dt;
        self.rms_t += dt * self.rms_scale * audio.map_or(1.0, |a| 600.0 * a.rms());
        self.mul_t += dt * self.mul_scale;
    }

    pub fn t(&self) -> f32 { self.t }

    pub fn t_rms(&self) -> f32 { self.rms_t }
    pub fn t_rms_scale(&mut self, scale: f32) { self.rms_scale = scale; }

    pub fn t_mul(&self) -> f32 { self.mul_t }
    pub fn t_mul_scale(&mut self, scale: f32) { self.mul_scale = scale; }
}