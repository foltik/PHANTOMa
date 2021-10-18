use lib::audio::Audio;
use lib::time::{BeatClock, BeatDetect};

use super::mixxx::Mixxx;

pub enum BeatSource {
    Detect,
    Clock,
}

pub struct Beat {
    detect: BeatDetect,
    clock: BeatClock,
    source: BeatSource,
    active: bool,
    manual: bool,
}

impl Beat {
    pub fn midi(&mut self, msg: Midi) {
        match msg {
            // Hold or toggle active with two bank buttons
            Midi::BankButton(_, b) => self.active = b,

            // Clock multipliers, sync, and source control
            Midi::CtrlButton(0, true) => self.clock.sync(),
            Midi::CtrlButton(1, true) => self.clock.mul = 2.0,
            Midi::CtrlButton(2, true) => self.clock.mul = 1.0,
            Midi::CtrlButton(3, true) => self.clock.mul = 0.5,
            Midi::CtrlButton(4, true) => self.clock.mul = 0.25,
            Midi::CtrlButton(5, true) => self.source = BeatSource::Clock,
            Midi::CtrlButton(5, false) => self.source = BeatSource::Detect,

            // Detector tuning
            Midi::Knob(7, f) => self.detect.bpm_max = 200.0 + f * 300.0,
            Midi::Knob(8, f) => self.detect.thres = 0.1 * f,

            _ => {}
        }
    }

    pub fn beat(&mut self) {
        self.manual = true;
    }

    pub fn update(&mut self, dt: f32, audio: &mut Audio, mixxx: &Mixxx) -> bool {
        mixxx
            .decks
            .iter()
            .filter(|d| d.active)
            .take(1)
            .for_each(|d| self.clock.bpm = d.bpm);

        let detect = self.detect.update(dt, audio);
        let clock = self.clock.update(dt);
        let manual = self.manual;
        self.manual = false;

        let beat = match self.source {
            BeatSource::Detect => detect,
            BeatSource::Clock => clock,
        };

        // self.active && (beat || manual)

        manual
    }
}

impl Default for Beat {
    fn default() -> Self {
        Beat {
            detect: BeatDetect::new(40.0, 120.0, 0.005, 400.0),
            clock: BeatClock::new(60.0),
            source: BeatSource::Detect,
            active: true,
            manual: false,
        }
    }
}
