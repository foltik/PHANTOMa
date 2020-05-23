use nannou::prelude::*;
use nannou::ui::prelude::*;
use lib::{
    audio::{self, Audio},
    osc::{Osc, OscMessage},
    midi::{Midi, MidiMessage},
    time::{DecayEnv, BeatDetect, BeatClock}
};

fn main() {
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .run();
}

struct Model {
    ui: Ui,
    ids: Ids,

    audio: Box<dyn Audio>,
    midi: Midi,
    osc: Osc,

    beat_detect: BeatDetect,
    beat_clock: BeatClock,
    beat_pause: bool,
    beat_net: bool,

    decay: DecayEnv,

    t: f32,
}

struct Ids {
    f0: widget::Id,
    f1: widget::Id,
    thres: widget::Id,
    bpm_max: widget::Id,
}

fn model(app: &App) -> Model {
    let mut ui = app.new_ui().build().unwrap();

    let ids = Ids {
        f0: ui.generate_widget_id(),
        f1: ui.generate_widget_id(),
        thres: ui.generate_widget_id(),
        bpm_max: ui.generate_widget_id(),
    };

    let audio = Box::new(audio::init());
    let osc = Osc::init(34254);
    let midi = Midi::init();

    let beat_detect = BeatDetect::new(40.0, 120.0, 0.005, 200.0);
    let beat_clock = BeatClock::new(120.0);

    let decay = DecayEnv::new()
        .with("beat", 340.0);

    Model {
        ui,
        ids,

        audio,
        midi,
        osc,

        beat_detect,
        beat_clock,
        beat_pause: false,
        beat_net: false,

        decay,

        t: 0.0,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    model.audio.update();

    for (_bank, message) in model.midi.poll() {
        match message {
            MidiMessage::CtrlButton(3, t) => model.beat_pause = t,
            MidiMessage::CtrlButton(4, t) => model.beat_pause = t,
            MidiMessage::CtrlButton(5, t) => model.beat_net = t,
            MidiMessage::MainButton(0, true) => model.beat_clock.sync(),
            MidiMessage::Knob(6, f) => model.decay.t("beat", 50.0 + f * 200.0),
            MidiMessage::Knob(7, f) => model.beat_detect.bpm_max = 200.0 + f * 200.0,
            MidiMessage::Knob(8, f) => model.beat_detect.thres = 0.1 * f,
            _ => {}
        }
    }

    for msg in model.osc.poll() {
        match msg {
            OscMessage::Bpm(bpm) => model.beat_clock.bpm = bpm,
            _ => {}
        }
    }


    model.t += model.audio.rms();

    let ms = update.since_last.as_nanos() as f32 / 1_000_000.0;
    model.decay.update(ms);

    let audio_beat = model.beat_detect.update(ms, &*model.audio);
    let clock_beat = model.beat_clock.update(ms);


    let beat = if model.beat_net { clock_beat } else { audio_beat };

    if beat && !model.beat_pause {
        model.decay.set("beat");
    }

    let ui = &mut model.ui.set_widgets();

    fn slider(v: f32, min: f32, max: f32) -> widget::Slider<'static, f32> {
        widget::Slider::new(v, min, max)
            .w_h(200.0, 30.0)
            .label_font_size(15)
            .rgb(0.3, 0.3, 0.3)
            .label_rgb(1.0, 1.0, 1.0)
            .border(0.0)
    }

    for v in slider(model.beat_detect.f0, 0.0, 1000.0)
        .top_left_with_margin(20.0)
        .label(&format!("f0: {}Hz [{}]", model.beat_detect.f0, audio::bin(model.beat_detect.f0)))
        .set(model.ids.f0, ui)
    {
        model.beat_detect.f0 = v;
    }

    for v in slider(model.beat_detect.f1, 0.0, 1000.0)
        .down(10.0)
        .label(&format!("f1: {}Hz [{}]", model.beat_detect.f1, audio::bin(model.beat_detect.f1)))
        .set(model.ids.f1, ui)
    {
        model.beat_detect.f1 = v;
    }

    for v in slider(model.beat_detect.thres, 0.001, 0.1)
        .down(10.0)
        .label(&format!("thres: {}", model.beat_detect.thres))
        .set(model.ids.thres, ui)
    {
        model.beat_detect.thres = v;
    }

    for v in slider(model.beat_detect.bpm_max, 200.0, 400.0)
        .down(10.0)
        .label(&format!("bpm_max: {}", model.beat_detect.bpm_max))
        .set(model.ids.bpm_max, ui)
    {
        model.beat_detect.bpm_max = v;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background()
        .color(BLACK);

    draw.ellipse()
        .y(200.0)
        .x(400.0)
        .color(WHITE)
        .radius(100.0 * model.decay.v("beat"));

    draw.ellipse()
        .y(200.0)
        .x(-400.0)
        .color(WHITE)
        .radius(100.0 * model.decay.v("beat"));

    draw.rect()
        .w_h(500.0, 500.0)
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0 + 4.0 * model.decay.v("beat"))
        .rotate(model.t * TAU);

    draw.rect()
        .w_h(500.0 / 2.0f32.sqrt(), 500.0 / 2.0f32.sqrt())
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0 + 4.0 * model.decay.v("beat"))
        .rotate(model.t * -TAU + TAU / 4.0);

    draw.text(&format!("RMS: {:.5}", model.audio.rms()))
        .y(420.0)
        .color(WHITE);

    draw.text(&format!("Peak dBFS: {:.2}", audio::dbfs(model.audio.peak())))
        .y(400.0)
        .color(WHITE);

    draw.text(&format!("OSC BPM: {:.2}", model.beat_clock.bpm))
        .y(380.0)
        .color(WHITE);

    //let scale = |x: usize| ((x + 1) as f32).log10() / ((audio::FFT_SIZE + 1) as f32).log10();
    let scale = |x: usize| ((x + 1) as f32).log10() / ((100 + 1) as f32).log10();

    let y = -400.0;
    let (w, h) = (1800.0, 800.0);
    let pts: Vec<_> = model.audio.fft().iter().enumerate().map(|(i, s)| pt2(w * scale(i) - w / 2.0, s * h + y)).collect();

    let samples: Vec<_> = model.audio.samples().iter().enumerate().map(|(i, s)| pt2(w * (i as f32 / audio::FFT_FSIZE) - w / 2.0, s * 100.0 - 100.0)).collect();

    draw.polyline()
        .color(WHITE)
        .weight(1.0)
        .join_round()
        .points(samples.iter().map(|p| *p));

    draw.polyline()
        .color(WHITE)
        .weight(1.0)
        .join_round()
        .points(pts.iter().take(100).map(|p| *p));

    for (i, p) in pts.iter().take(20).enumerate() {
        draw.line()
            .color(WHITE)
            .stroke_weight(1.0)
            .start(pt2(p.x, -410.0))
            .end(pt2(p.x, -420.0));

        draw.text(&format!("{:.2}", audio::freq(i)))
            .x_y(p.x, -450.0)
            .rotate(TAU / 4.0)
            .color(WHITE)
            .font_size(8);
    }

    draw.to_frame(app, &frame).unwrap();
    model.ui.draw_to_frame(app, &frame).unwrap();
}
