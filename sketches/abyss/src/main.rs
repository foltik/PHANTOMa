// use gfx::animation::Animator;
// use gfx::pass::{FilterPass, RingPass, TextPass, TextPassBuilder, ImagePass};
// use gfx::scene::Scene;
use lib::prelude::*;
use lib::midi2::Midi;
use lib::midi2::device::launchpad_x::{self, LaunchpadX};
use lib::midi2::device::worlde_easycontrol9::{self, WorldeEasyControl9};

// use lib::resource;

mod pipeline;
use pipeline::*;

mod time;
use time::Time;

// mod beat;
// use beat::Beat;

mod stage;
use stage::Stage;

mod test; use test::Tester;

// # boot up sequence
//   ## loading
//     - loading circle on main disp, glitch
//     - rotating white upper lights
//   ## init
//     - code starts scrolling down screens
//   ## power
//     - all turns blue
//     - lines start rotating around pipes'

// # tunnel sequence
//   ## torus
//   ## movement on sides

// # wave sequence
//   ## isotri
//   ## chladni
//   ## space gif
//   ## bubbles
//   ## inner torus
// 
//   ## contour lines on alt

// # space sequence
//   ## starfield on panels

// # magic sequence
//   ## glyph center
//   ## scrolling prime and twin prime wings

// # green cyber sequence
//   ## bad tv shader

// # orange numbers sequence
//   ## neon squares?
//   ## nixie flashing

// # outro
//   - rotating lines on pipes fades to white
//   - white slowly starts to turn to static
//   - TV off animation to black

fn main() {
    lib::app::run(model, input, update, view);
}

pub struct Model {
    // audio: Option<Audio>,
    ctrl: Option<Midi<WorldeEasyControl9>>,
    pad: Option<Midi<LaunchpadX>>,

    t: Time,
    fx: FxPass,

    stencil: StencilPass,
    stage: Option<Stage>,
    test: Tester,
}

impl Model {
    pub fn t(&self) -> f32 {
        self.t.t_rms()
    }

    pub fn tc(&self) -> f32 {
        self.t.t()
    }
}

async fn model(app: &App) -> Model {
    let device = &app.device;

    // let audio = Audio::maybe_open();
    // let audio = None;
    let ctrl = Midi::<WorldeEasyControl9>::maybe_open("WORLDE easy control");
    let pad = Midi::<LaunchpadX>::maybe_open("Launchpad X LPX MIDI");
    if let Some(pad) = pad.as_ref() {
        use launchpad_x::{*, types::*};
        pad.send(Output::Mode(Mode::Programmer)).await;
        pad.send(Output::Pressure(Pressure::Off, PressureCurve::Medium)).await;
        pad.send(Output::Clear).await;
        for i in 0..8 {
            pad.send(Output::Light(Coord(i, 8).into(), Color::Index(1))).await;
            pad.send(Output::Light(Coord(8, i).into(), Color::Index(1))).await;
        }
    } else {
        log::info!("pad none");
    }

    let stencil = StencilPass::new(device);
    let stage = Stage::new(device, &stencil);

    Model {
        // audio,
        ctrl,
        pad,

        t: Time::new(),
        fx: FxPass::new(device, (1920, 1080)),

        stencil,
        stage: Some(stage),
        test: Tester::new(device),
    }
}

async fn input(app: &App, m: &mut Model, state: KeyState, key: Key) {
    if state == KeyState::Pressed { return; }
    m.stage = Some(m.stage.take().unwrap().key(app, m, key).await);
    m.test.key(key).await;
}

async fn ctrl(app: &App, m: &mut Model, input: worlde_easycontrol9::Input) {
    m.fx.ctrl(input);
    m.stage = Some(m.stage.take().unwrap().ctrl(app, m, input).await);
    m.test.ctrl(input).await;
}

async fn pad(app: &App, m: &mut Model, input: launchpad_x::Input) {
    m.stage = Some(m.stage.take().unwrap().pad(app, m, input).await);
    m.test.pad(input).await;

    use launchpad_x::{*, types::*};
    if let Some((i, c)) = match input {
        Input::Up(true)      => Some((0,  Coord(0, 8))),
        Input::Down(true)    => Some((1,  Coord(1, 8))),
        Input::Left(true)    => Some((2,  Coord(2, 8))),
        Input::Right(true)   => Some((3,  Coord(3, 8))),
        Input::Session(true) => Some((4,  Coord(4, 8))),
        Input::Note(true)    => Some((5,  Coord(5, 8))),
        Input::Custom(true)  => Some((6,  Coord(6, 8))),
        Input::Capture(true) => Some((7,  Coord(7, 8))),
        Input::Volume(true)  => Some((8,  Coord(8, 7))),
        Input::Pan(true)     => Some((9,  Coord(8, 6))),
        Input::A(true)       => Some((10, Coord(8, 5))),
        Input::B(true)       => Some((11, Coord(8, 4))),
        Input::Stop(true)    => Some((12, Coord(8, 3))),
        Input::Mute(true)    => Some((13, Coord(8, 2))),
        Input::Solo(true)    => Some((14, Coord(8, 1))),
        Input::Record(true)  => Some((15, Coord(8, 0))),
        _ => None
    } {
        m.stage = Some(m.stage.take().unwrap().go(app, m, i).await);

        let pad = m.pad.as_ref().unwrap();
        for i in 0..8 {
            pad.send(Output::Light(Coord(i, 8).into(), Color::Index(1))).await;
            pad.send(Output::Light(Coord(8, i).into(), Color::Index(1))).await;
        }
        pad.send(Output::Light(c.into(), Color::Magenta)).await;
    }
}

async fn update(app: &App, m: &mut Model, dt: f32) {
    // if let Some(audio) = m.audio.as_mut() { audio.update(); }

    // m.t.update(dt, m.audio.as_ref());
    m.t.update(dt, None);

    if let Some(dev) = m.ctrl.as_ref() { 
        for i in dev.recv() {
            ctrl(app, m, i).await;
        }
    }
    if let Some(dev) = m.pad.as_ref() { 
        for i in dev.recv() {
            pad(app, m, i).await;
        }
    }

    // m.stage = Some(m.stage.take().unwrap().update(app, m, dt));
    // m.stencil.update(&*m.fx);

    m.test.update(m.pad.as_ref(), dt).await;
}

fn view(_app: &App, m: &mut Model, frame: &mut Frame, view: &wgpu::RawTextureView) {
    // m.stage = Some(m.stage.take().unwrap().encode(frame, &m.stencil));
    // m.stencil.encode(frame, view);

    m.test.encode(frame, view);
}
