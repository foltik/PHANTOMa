use nannou::prelude::*;
use nannou::ui::prelude::*;
use lib::{BeatDecay, audio, audio::Audio, osc::{Osc, OscMessage}, midi::{Midi, MidiMessage}};

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
    beat: BeatDecay,
    sync: bool,
    toggle: bool,
    acc: f32,
    offset: f32,
    bpm: f32,
    t: f32,
}

struct Ids {
    f0: widget::Id,
    f1: widget::Id,
    thres: widget::Id,
    bpm: widget::Id,
    overlap: widget::Id,
}

fn model(app: &App) -> Model {
    let mut ui = app.new_ui().build().unwrap();

    let ids = Ids {
        f0: ui.generate_widget_id(),
        f1: ui.generate_widget_id(),
        thres: ui.generate_widget_id(),
        bpm: ui.generate_widget_id(),
        overlap: ui.generate_widget_id(),
    };

    let audio = Box::new(audio::init());
    let osc = Osc::init(34254);
    let midi = Midi::init();

    let beat = BeatDecay::new(40.0, 120.0, 0.005, false, 250.0);

    Model {
        ui,
        ids,
        audio,
        midi,
        osc,
        beat,
        toggle: false,
        acc: 0.0,
        sync: false,
        offset: 0.0,
        bpm: 250.0,
        t: 0.0,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let ui = &mut model.ui.set_widgets();

    let ms = update.since_last.as_nanos() as f32 / 1_000_000.0;
    model.acc += ms;

    for (_, message) in model.midi.poll() {
        match message {
            MidiMessage::MainButton(0, t) => model.toggle = t,
            MidiMessage::MainButton(1, true) => model.sync = true,
            _ => {}
        }
    }

    let next_beat = (1.0 / model.bpm) * 60.0 * 1000.0;
    if model.acc >= next_beat && !model.toggle {
        model.acc = model.acc - next_beat;
        model.beat.decay.set_max();
    }

    if (model.sync) {
        model.sync = false;
        model.acc = 0.0;
        model.beat.decay.set_max();
    }

    for msg in model.osc.poll() {
        match msg {
            OscMessage::Bpm(bpm) => model.bpm = bpm,
            _ => {}
        }
    }

    fn slider(v: f32, min: f32, max: f32) -> widget::Slider<'static, f32> {
        widget::Slider::new(v, min, max)
            .w_h(200.0, 30.0)
            .label_font_size(15)
            .rgb(0.3, 0.3, 0.3)
            .label_rgb(1.0, 1.0, 1.0)
            .border(0.0)
    }

    for v in slider(model.beat.f0, 0.0, 1000.0)
        .top_left_with_margin(20.0)
        .label(&format!("f0: {}Hz [{}]", model.beat.f0, audio::bin(model.beat.f0)))
        .set(model.ids.f0, ui)
    {
        model.beat.f0 = v;
    }

    for v in slider(model.beat.f1, 0.0, 1000.0)
        .down(10.0)
        .label(&format!("f1: {}Hz [{}]", model.beat.f1, audio::bin(model.beat.f1)))
        .set(model.ids.f1, ui)
    {
        model.beat.f1 = v;
    }

    for v in slider(model.beat.thres, 0.001, 0.1)
        .down(10.0)
        .label(&format!("thres: {}", model.beat.thres))
        .set(model.ids.thres, ui)
    {
        model.beat.thres = v;
    }

    for v in slider(model.offset, -100.0, 100.0)
        .down(10.0)
        .label(&format!("offset: {}", model.offset))
        .set(model.ids.bpm, ui)
    {
        //model.bpm = v;
        //model.beat.sens = 1.0 / (v / 60.0) * 1000.0;
        model.offset = v;
    }

    for v in widget::Toggle::new(model.beat.overlap)
        .down(10.0)
        .label("overlap")
        .set(model.ids.overlap, ui)
    {
        //model.beat.overlap = v;
        model.sync = true;
    }

    model.audio.update();

    let ms = update.since_last.as_nanos() as f32 / 1_000_000.0;

    model.beat.update(ms, &*model.audio);
    model.t += model.audio.rms();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background()
        .color(BLACK);

    draw.ellipse()
        .y(200.0)
        .x(400.0)
        .color(WHITE)
        .radius(100.0 * model.beat.v());

    draw.ellipse()
        .y(200.0)
        .x(-400.0)
        .color(WHITE)
        .radius(100.0 * model.beat.v());

    draw.rect()
        .w_h(500.0, 500.0)
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0 + 4.0 * model.beat.v())
        .rotate(model.t * TAU);

    draw.rect()
        .w_h(500.0 / 2.0f32.sqrt(), 500.0 / 2.0f32.sqrt())
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0 + 4.0 * model.beat.v())
        .rotate(model.t * -TAU + TAU / 4.0);

    draw.text(&format!("RMS: {:.5}", model.audio.rms()))
        .y(420.0)
        .color(WHITE);

    draw.text(&format!("Peak dBFS: {:.2}", audio::dbfs(model.audio.peak())))
        .y(400.0)
        .color(WHITE);

    draw.text(&format!("BPM: {:.2}", model.bpm))
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
