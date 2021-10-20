use lib::prelude::*;
use lib::midi2::device::launchpad_x::{self, LaunchpadX};
use lib::midi2::device::worlde_easycontrol9::{self, WorldeEasyControl9};

fn main() {
    lib::app::run(model, input, update, view);
}
pub struct Model {
    ctrl: Option<Midi<WorldeEasyControl9>>,
    pad: Option<Midi<LaunchpadX>>,
}

fn model(app: &App) -> Model {
    let device = &app.device;

    let ctrl = Midi::<WorldeEasyControl9>::maybe_open("WORLDE easy control");
    let pad = Midi::<LaunchpadX>::maybe_open("Launchpad X LPX MIDI");
    if let Some(pad) = pad.as_ref() {
        use launchpad_x::{*, types::*};
        pad.send(Output::Mode(Mode::Programmer));
        pad.send(Output::Pressure(Pressure::Off, PressureCurve::Medium));
        pad.send(Output::Clear);
        for i in 0..8 {
            pad.send(Output::Light(Coord(i, 8).into(), Color::Index(1)));
            pad.send(Output::Light(Coord(8, i).into(), Color::Index(1)));
        }
    }

    Model {
        ctrl,
        pad,
    }
}

fn input(app: &App, m: &mut Model, state: KeyState, key: Key) {
    log::info!("input: {:?} -> {:?}", key, state);
}

fn update(app: &App, m: &mut Model, dt: f32) {
    if let Some(dev) = m.ctrl.as_ref() { 
        dev.recv().for_each(|i| log::info!("ctrl: {:?}", i));
    }
    if let Some(dev) = m.pad.as_ref() {
        dev.recv().for_each(|i| log::info!("pad: {:?}", i));
    }
}

fn view(app: &App, m: &mut Model, frame: &mut Frame, view: &wgpu::RawTextureView) {
}
