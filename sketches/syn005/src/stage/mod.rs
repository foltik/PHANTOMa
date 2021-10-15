use lib::prelude::*;

use crate::Model;

// mod city;
// use city::City;

mod lobby;
use lobby::Lobby;

mod bhop;
use bhop::Bhop;

mod core;
use self::core::Core;

mod canals;
use canals::Canals;

mod medsci;
use medsci::MedSci;

mod shodan;
use shodan::Shodan;

mod stasis;
use stasis::Stasis;

pub enum Set {
    Lobby,
    Bhop,
    Canals,
    Core,
    MedSci,
    Shodan,
    Stasis,
}

impl Set {
    pub fn next(&self) -> Self {
        match self {
            Set::Lobby => Set::Bhop,
            Set::Bhop => Set::Canals,
            Set::Canals => Set::Core,
            Set::Core => Set::MedSci,
            Set::MedSci => Set::Shodan,
            Set::Shodan => Set::Stasis,
            Set::Stasis => Set::Stasis,
        }
    }
}

pub struct Stage {
    pub set: Set,

    lobby: Lobby,
    bhop: Bhop,
    canals: Canals,
    core: Core,
    medsci: MedSci,
    shodan: Shodan,
    stasis: Stasis,
}

impl Stage {
    pub fn new(app: &App) -> Self {
        Self {
            set: Set::Lobby,

            lobby: Lobby::new(app),
            bhop: Bhop::new(app),
            canals: Canals::new(app),
            core: Core::new(app),
            medsci: MedSci::new(app),
            shodan: Shodan::new(app),
            stasis: Stasis::new(app),
        }
    }

    pub fn next(&mut self) {
        self.set = self.set.next();
    }

    pub fn input(mut self, app: &App, model: &mut Model, state: KeyState, key: Key) -> Self {
        match self.set {
            Set::Lobby => self.lobby.input(app, model, state, key),
            Set::Bhop => self.bhop.input(app, model, state, key),
            Set::Canals => self.canals.input(app, model, state, key),
            Set::Core => self.core.input(app, model, state, key),
            Set::MedSci => self.medsci.input(app, model, state, key),
            Set::Shodan => self.shodan.input(app, model, state, key),
            Set::Stasis => self.stasis.input(app, model, state, key),
            _ => {},
        }
        self
    }

    pub fn midi(mut self, model: &mut Model, bank: MidiBank, msg: Midi) -> Self {
        match self.set {
            Set::Lobby => self.lobby.midi(model, bank, msg),
            Set::Bhop => self.bhop.midi(model, bank, msg),
            Set::Canals => self.canals.midi(model, bank, msg),
            Set::Core => self.core.midi(model, bank, msg),
            Set::MedSci => self.medsci.midi(model, bank, msg),
            Set::Shodan => self.shodan.midi(model, bank, msg),
            Set::Stasis => self.stasis.midi(model, bank, msg),
            _ => {},
        }
        self
    }

    pub fn update(mut self, app: &App, model: &mut Model, dt: f32) -> Self {
        match self.set {
            Set::Lobby => self.lobby.update(app, model, dt),
            Set::Bhop => self.bhop.update(app, model, dt),
            Set::Canals => self.canals.update(app, model, dt),
            Set::Core => self.core.update(app, model, dt),
            Set::MedSci => self.medsci.update(app, model, dt),
            Set::Shodan => self.shodan.update(app, model, dt),
            Set::Stasis => self.stasis.update(app, model, dt),
            _ => {}
        }
        self
    }

    pub fn view(mut self, frame: &mut Frame, target: &wgpu::RawTextureView) -> Self {
        match self.set {
            Set::Lobby => self.lobby.view(frame, target),
            Set::Bhop => self.bhop.view(frame, target),
            Set::Canals => self.canals.view(frame, target),
            Set::Core => self.core.view(frame, target),
            Set::MedSci => self.medsci.view(frame, target),
            Set::Shodan => self.shodan.view(frame, target),
            Set::Stasis => self.stasis.view(frame, target),
            _ => {}
        }
        self
    }
}