use nannou::math::cgmath::{Point3, Vector3};
use nannou::prelude::*;
use nannou::ui::prelude::*;
use nannou::text::{font, Font};

use lib::{
    *,
    audio::{self, Audio},
    midi::{Midi, MidiMessage},
    gfx::{Camera, CameraDesc, CameraUniform, Effect, Mesh, Present, Uniform, Drawer},
};

fn main() {
    lib::init_logging(2);
    nannou::app(model).update(update).view(view).run();
}

// TODO: this is mostly an Effect with two input images and a hard coded
// shader that sums the two image values.
// I only made it because for some reason a Draw clears out the image from Maze
// before drawing, so either figure out how to genericize Effect over N input
// images, or fix Draw clearing the image.
pub struct Composite {
    view1: wgpu::TextureView,
    view2: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl Composite {
    fn new(device: &wgpu::Device) -> Self {
        let vs_mod = read_shader(device, gfx::BILLBOARD_SHADER);
        let fs_mod = read_shader(device, "add.frag.spv");

        let tex1 = gfx::texture_builder()
            .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
            .build(device);
        let view1 = tex1.view().build();

        let tex2 = gfx::texture_builder()
            .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
            .build(device);
        let view2 = tex2.view().build();

        let sampler = wgpu::SamplerBuilder::new().build(&device);

        let bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
            .sampled_texture_from(wgpu::ShaderStage::FRAGMENT, &tex1)
            .sampled_texture_from(wgpu::ShaderStage::FRAGMENT, &tex2)
            .sampler(wgpu::ShaderStage::FRAGMENT)
            .build(device);

        let bind_group = wgpu::BindGroupBuilder::new()
            .texture_view(&view1)
            .texture_view(&view2)
            .sampler(&sampler)
            .build(device, &bind_group_layout);

        let pipeline_layout = wgpu::create_pipeline_layout(device, &[&bind_group_layout]);
        let pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
            .fragment_shader(&fs_mod)
            .color_format(gfx::TEXTURE_FORMAT)
            .build(device);

        Self {
            view1,
            view2,
            bind_group,
            pipeline,
        }
    }

    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let mut pass = wgpu::RenderPassBuilder::new()
            .color_attachment(target, |c| c)
            .begin(encoder);

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);

        pass.draw(0..3, 0..1);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct PointLight {
    pos: Point3<f32>,
    constant: f32,
    linear: f32,
    quadratic: f32,
    ambient: f32,
    diffuse: f32,
    specular: f32,
}

// Vertex, Texture, Normal
struct MazeVertex;
impl wgpu::VertexDescriptor for MazeVertex {
    const STRIDE: wgpu::BufferAddress = std::mem::size_of::<VertTexNorm>() as wgpu::BufferAddress;
    const ATTRIBUTES: &'static [wgpu::VertexAttributeDescriptor] = &[
        wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float3,
            offset: 0,
            shader_location: 0,
        },
        wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float2,
            offset: 3 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
            shader_location: 1,
        },
        wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float3,
            offset: 5 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
            shader_location: 2,
        },
    ];
}

struct Maze {
    floor: Mesh,
    wall: Mesh,

    depth: wgpu::TextureView,

    camera: Camera,
    lights: Uniform<PointLight>,

    vertex_group: wgpu::BindGroup,
    floor_group: wgpu::BindGroup,
    wall_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl Maze {
    fn new(device: &wgpu::Device, window: &Window, encoder: &mut wgpu::CommandEncoder) -> Self {
        let vs_mod = read_shader(device, "maze.vert.spv");
        let fs_mod = read_shader(device, "maze.frag.spv");

        let mut objects = read_model("maze-tex.obj");

        let floor_data = objects.remove(0).meshes.remove(0);
        let wall_data = objects.remove(0).meshes.remove(0);

        let floor = Mesh::new(device, window, encoder, &floor_data);
        let wall = Mesh::new(device, window, encoder, &wall_data);

        let camera = Camera::new(
            device,
            CameraDesc {
                eye: (0.0, 0.75, -2.0).into(),
                target: (0.0, 0.75, 0.0).into(),
                up: -Vector3::unit_y(),
                fov: 90.0,
                near: 0.1,
                far: 100.0,
            },
        );

        let lights = Uniform::new(device);

        let vertex_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStage::VERTEX, false)
            .build(device);

        let vertex_group = wgpu::BindGroupBuilder::new()
            .buffer::<CameraUniform>(camera.buffer(), 0..1)
            .build(device, &vertex_layout);

        let wall_sampler = wgpu::SamplerBuilder::new()
            .mag_filter(wgpu::FilterMode::Nearest)
            .address_mode(wgpu::AddressMode::Repeat)
            .build(&device);

        let floor_sampler = wgpu::SamplerBuilder::new()
            .mag_filter(wgpu::FilterMode::Nearest)
            .address_mode(wgpu::AddressMode::Repeat)
            .build(&device);

        let dim = wgpu::TextureViewDimension::D2;

        let tex_layout = wgpu::BindGroupLayoutBuilder::new()
            .sampled_texture(wgpu::ShaderStage::FRAGMENT, false, dim)
            .sampler(wgpu::ShaderStage::FRAGMENT)
            .uniform_buffer(wgpu::ShaderStage::FRAGMENT, false)
            .build(device);

        let floor_group = wgpu::BindGroupBuilder::new()
            .texture_view(&floor.texture.clone().unwrap())
            .sampler(&floor_sampler)
            .buffer::<PointLight>(lights.buffer(), 0..1)
            .build(device, &tex_layout);

        let wall_group = wgpu::BindGroupBuilder::new()
            .texture_view(&wall.texture.clone().unwrap())
            .sampler(&wall_sampler)
            .buffer::<PointLight>(lights.buffer(), 0..1)
            .build(device, &tex_layout);

        let depth = gfx::depth_builder().build(device);
        let depth_view = depth.view().build();

        let pipeline_layout = wgpu::create_pipeline_layout(device, &[&vertex_layout, &tex_layout]);
        let pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
            .fragment_shader(&fs_mod)
            .color_format(gfx::TEXTURE_FORMAT)
            .add_vertex_buffer::<MazeVertex>()
            .depth_format(gfx::DEPTH_FORMAT)
            .build(device);

        Self {
            floor,
            wall,

            depth: depth_view,

            camera,
            lights,

            vertex_group,
            floor_group,
            wall_group,
            pipeline,
        }
    }

    fn update(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        light: &PointLight,
    ) {
        self.camera.update(device, encoder);
        self.lights.upload(device, encoder, light.clone());
    }

    fn encode(&self, encoder: &mut wgpu::CommandEncoder, texture: &wgpu::TextureView) {
        let mut pass = wgpu::RenderPassBuilder::new()
            .color_attachment(texture, |c| c)
            .depth_stencil_attachment(&self.depth, |d| d)
            .begin(encoder);

        pass.set_bind_group(0, &self.vertex_group, &[]);
        pass.set_pipeline(&self.pipeline);

        pass.set_bind_group(1, &self.floor_group, &[]);
        pass.set_vertex_buffers(0, &[(&self.floor.buffer, 0)]);
        pass.draw(0..self.floor.len as u32, 0..1);

        pass.set_bind_group(1, &self.wall_group, &[]);
        pass.set_vertex_buffers(0, &[(&self.wall.buffer, 0)]);
        pass.draw(0..self.wall.len as u32, 0..1);
    }
}

// Uniform to control the Glitch shader parameters
#[derive(Copy, Clone, Debug)]
struct EffectState {
    t: f32,
    pause: f32,
    glitch: f32,
    glitch_mo: f32,
    red: f32,
}

// Widget IDs for the tweaking UI
struct Ids {
    constant: widget::Id,
    linear: widget::Id,
    quadratic: widget::Id,
    ambient: widget::Id,
    diffuse: widget::Id,
    specular: widget::Id,
}

struct Model {
    audio: Box<dyn Audio>,
    midi: Midi,

    font: Font,

    beat: BeatDecay,
    t: f32,
    t_pause: f32,
    red: f32,
    decay_red: Decay,
    decay_fov: Decay,
    decay_glitch: Decay,
    light: PointLight,
    glitch_state: EffectState,

    maze: Maze,
    drawer: Drawer,
    composite: Composite,
    glitch: Effect<EffectState>,
    present: Present,

    ids: Ids,
    ui: Ui,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .size(1920, 1080)
        .title("PHANTOMa")
        .view(view)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let device = window.swap_chain_device();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    let state = EffectState {
        t: 0.0,
        pause: 0.0,
        glitch: 0.0,
        glitch_mo: 0.0,
        red: 0.0,
    };

    let maze = Maze::new(device, &window, &mut encoder);
    let glitch = Effect::new(device, "glitch.frag.spv");
    let present = Present::new(device, window.msaa_samples());

    let mut ui = app.new_ui().build().unwrap();
    let ids = Ids {
        constant: ui.generate_widget_id(),
        linear: ui.generate_widget_id(),
        quadratic: ui.generate_widget_id(),
        ambient: ui.generate_widget_id(),
        diffuse: ui.generate_widget_id(),
        specular: ui.generate_widget_id(),
    };

    let light = PointLight {
        pos: Point3::new(0.0, 0.5, 3.0),
        constant: 0.0,
        linear: 1.325,
        quadratic: 0.55,
        ambient: 1.5,
        diffuse: 7.5,
        specular: 1.5,
    };

    window
        .swap_chain_queue()
        .lock()
        .unwrap()
        .submit(&[encoder.finish()]);

    Model {
        audio: Box::new(audio::init()),
        midi: Midi::init(),

        font: font::from_file("../../resources/fonts/magi.ttf").unwrap(),

        beat: BeatDecay::new(40.0, 120.0, 0.005, false, 250.0),
        t: 0.0,
        t_pause: 0.0,
        red: 0.0,
        decay_red: Decay::new(),
        decay_fov: Decay::new(),
        decay_glitch: Decay::new(),
        light,
        glitch_state: state,

        maze,
        drawer: Drawer::new(device, 4),
        composite: Composite::new(device),
        glitch,
        present,

        ui,
        ids,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let ms = update.since_last.as_nanos() as f32 / 1_000_000.0;

    model.audio.update();
    model.beat.update(ms, &*model.audio);

    model.decay_glitch.update(ms / 100.0);
    model.decay_red.update(ms / 250.0);
    model.decay_fov.update(ms / 1500.0);


    let t_mod = 50.0 * ms * model.audio.rms();

    model.t += t_mod;
    model.t_pause += t_mod * (1.0 - model.glitch_state.pause);
    model.glitch_state.t = model.t;


    model.glitch_state.red = (1.0 + model.red * 2.0) * model.decay_red.v();


    let cam = &mut model.maze.camera.desc;
    if model.decay_fov.is_zero() {
        cam.fov = 90.0 + 5.0 * model.decay_red.v();
    } else {
        cam.fov = 90.0 + 10.0 * (1.0 - model.decay_fov.v());
    }


    for (_, message) in model.midi.poll() {
        match message {
            MidiMessage::Knob(0, f) => model.glitch_state.pause = f,
            MidiMessage::Knob(1, f) => model.glitch_state.glitch = f,
            MidiMessage::Knob(2, f) => model.red = f,
            MidiMessage::MainButton(0, true) => model.decay_fov.set_max(),
            MidiMessage::MainButton(0, false) => model.decay_fov.set(0.0),
            MidiMessage::MainButton(1, true) => model.decay_red.set_max(),
            MidiMessage::MainButton(2, true) => model.decay_glitch.set_max(),
            _ => {}
        }
    }

    // -- Here be long and repetitive UI code --
    let ui = &mut model.ui.set_widgets();
    let slider = move |v: f32, min: f32, max: f32| -> widget::Slider<'static, f32> {
        widget::Slider::new(v, min, max)
            .w_h(200.0, 30.0)
            .label_font_size(15)
            .rgb(0.3, 0.3, 0.3)
            .label_rgb(1.0, 1.0, 1.0)
            .border(0.0)
    };

    let ids = &model.ids;
    let l = &mut model.light;
    for v in slider(l.constant, 0.0, 5.0)
        .top_left_with_margin(20.0)
        .label(&format!("constant: {}", l.constant))
        .set(ids.constant, ui)
    {
        l.constant = v;
    }

    for v in slider(l.linear, 0.0, 5.0)
        .down(10.0)
        .label(&format!("linear: {}", l.linear))
        .set(ids.linear, ui)
    {
        l.linear = v;
    }

    for v in slider(l.quadratic, 0.0, 5.0)
        .down(10.0)
        .label(&format!("quadratic: {}", l.quadratic))
        .set(ids.quadratic, ui)
    {
        l.quadratic = v;
    }

    for v in slider(l.ambient, 0.0, 300.0)
        .down(10.0)
        .label(&format!("ambient: {}", l.ambient))
        .set(ids.ambient, ui)
    {
        l.ambient = v;
    }

    for v in slider(l.diffuse, 0.0, 300.0)
        .down(10.0)
        .label(&format!("diffuse: {}", l.diffuse))
        .set(ids.diffuse, ui)
    {
        l.diffuse = v;
    }

    for v in slider(l.specular, 0.0, 300.0)
        .down(10.0)
        .label(&format!("specular: {}", l.specular))
        .set(ids.specular, ui)
    {
        l.specular = v;
    }
}

fn circle_pt(angle: f32) -> Point2 {
    pt2(angle.cos(), angle.sin())
}

fn poly_pts(n: i32, angle: f32) -> impl Iterator<Item = Point2> {
    (0..n).map(move |i| circle_pt(i as f32 / n as f32 * TAU + angle))
}

//#[rustfmt_skip]
fn demon1(draw: &Draw, font: Font, p: &Point2, r: f32, t: f32) {
    let draw = draw.translate(pt3(p.x, p.y, 0.0)).rotate(t / 1000.0);

    let tri0: Vec<_> = poly_pts(3, TAU / 4.0).map(|p| p * r).collect();
    let tri1: Vec<_> = poly_pts(3, -TAU / 4.0).map(|p| p * r).collect();

    // Outer ring
    draw.ellipse().radius(r).no_fill().stroke(WHITE).stroke_weight(1.0);

    // Second layer double ring
    draw.ellipse().radius(r * 0.83).no_fill().stroke(WHITE).stroke_weight(1.0);
    draw.ellipse().radius(r * 0.85).no_fill().stroke(WHITE).stroke_weight(1.0);

    // Outer triangles
    draw.polygon().no_fill().stroke(WHITE).stroke_weight(1.0)
        .points(tri0.iter().map(|p| *p));
    draw.polygon().no_fill().stroke(WHITE).stroke_weight(1.0)
        .points(tri1.iter().map(|p| *p));

    // Triangle point to center lines
    tri0.iter().chain(tri1.iter()).for_each(|p| {
        draw.line().color(WHITE).start(*p).end(pt2(0.0, 0.0));
    });

    // Triangle point rune circles
    tri0.iter().zip("AFH".chars()).for_each(|(p, ch)| {
        draw.ellipse().xy(*p).radius(r * 0.133).color(BLACK).stroke(WHITE).stroke_weight(1.0);
        draw.text(&ch.to_string()).font(font.clone())
            .font_size((r * 0.12).round() as u32)
            .xy(*p)
            .color(WHITE);
    });

    // Inner circle ring
    draw.ellipse().radius(r * 0.5).color(BLACK).stroke(WHITE).stroke_weight(1.0);

    // Inner runes
    poly_pts(26, t / 500.0)
        .map(|p| p * r * 0.465)
        .zip(lib::chars(1.0))
        .enumerate()
        .for_each(|(i, (p, ch))| {
            let angle = i as f32 / 30.0 * -TAU;
            draw.text(ch).font(font.clone())
                .font_size((r * 0.04).round() as u32)
                .xy(p).rotate(angle)
                .color(WHITE);
        });

    // Innner circle fill
    draw.ellipse().radius(r * 0.433).color(BLACK).stroke(WHITE).stroke_weight(1.0);

    // Inner tri
    draw.polygon().color(BLACK).stroke(WHITE).stroke_weight(1.0)
        .points(tri0.iter().map(|p| *p * 0.5));

    tri0.iter().for_each(|p| {
        // Outer fat circles
        draw.ellipse().xy(*p * 0.7).radius(r * 0.033).color(BLACK).stroke(WHITE).stroke_weight(2.0);

        // Inner fat circle lines
        draw.line().start(*p / 2.0).end(pt2(0.0, 0.0)).color(WHITE);

        // Inner fat circles
        draw.ellipse().xy(*p * 0.3).radius(r * 0.033).color(BLACK).stroke(WHITE).stroke_weight(4.0);
    });

    // Center outer, smaller, mid, point
    draw.ellipse().color(BLACK).stroke(WHITE).stroke_weight(1.0).radius(r * 0.2);
    draw.ellipse().no_fill().stroke(WHITE).stroke_weight(1.0).radius(r * 0.1733);
    draw.ellipse().no_fill().stroke(WHITE).stroke_weight(1.0).radius(r * 0.1);
    draw.ellipse().color(WHITE).radius(r * 0.033);
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Scope our draw code so we can clean it up and later render the UI, which needs to mutably
    // borrow the Frame as well
    {
        let draw = app.draw();
        draw.background().color(BLACK);
        demon1(&draw, model.font.clone(), &pt2(0.0, 0.0), 200.0 + 100.0 * model.decay_red.v(), model.t_pause);

        let window = app.main_window();
        let device = window.swap_chain_device();
        let mut encoder = frame.command_encoder();

        model.maze.update(device, &mut encoder, &model.light);
        model.maze.encode(&mut encoder, &model.composite.view1);
        model.drawer.encode(device, &mut encoder, &model.composite.view2, &draw);
        model.composite.encode(&mut encoder, model.glitch.view());
        model.glitch.update(device, &mut encoder, &model.glitch_state);
        model.glitch.encode(&mut encoder, model.present.view());

        model.present.encode(&mut encoder, &frame);
    }

    model.ui.draw_to_frame(app, &frame).unwrap();
}
