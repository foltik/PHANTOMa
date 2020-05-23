use nannou::geom::range::Range;
use nannou::math::cgmath::{Matrix4, Point3, Vector2, Vector3};
use nannou::prelude::*;
use nannou::text::{font, Font};
use nannou::ui::prelude::*;
use std::time::Instant;

use lib::{
    audio::{self, Audio},
    gfx::{Camera, CameraDesc, CameraUniform, Drawer, Effect, Mesh, Present, Uniform},
    interp::{self, Spline},
    midi::{Midi, MidiMessage},
    osc::{Osc, OscMessage},
    time::{BeatClock, BeatDetect, DecayEnv},
    *,
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
    pos: Vector4<f32>,
    ambient: Vector4<f32>,
    diffuse: Vector4<f32>,
    specular: Vector4<f32>,
    attenuation: Vector4<f32>,
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

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct MazeUniform {
    light: PointLight,
    eye: Vector3<f32>,
}

struct Maze {
    floor: Mesh,
    wall: Mesh,

    depth: wgpu::TextureView,

    camera: Camera,
    lights: Uniform<MazeUniform>,

    vertex_group: wgpu::BindGroup,
    floor_group: wgpu::BindGroup,
    wall_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl Maze {
    fn new(device: &wgpu::Device, window: &Window, encoder: &mut wgpu::CommandEncoder) -> Self {
        let vs_mod = read_shader(device, "maze.vert.spv");
        let fs_mod = read_shader(device, "maze.frag.spv");

        let mut maze_objs = read_model("maze-big.obj");

        let floor_data = maze_objs.remove(0).meshes.remove(0);
        let wall_data = maze_objs.remove(0).meshes.remove(0);

        let floor = Mesh::new(device, window, encoder, &floor_data);
        let wall = Mesh::new(device, window, encoder, &wall_data);

        let camera = Camera::new(
            device,
            CameraDesc {
                eye: (0.0, 0.75, 0.0).into(),
                target: (0.0, 0.75, 1.0).into(),
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
            .buffer::<MazeUniform>(lights.buffer(), 0..1)
            .build(device, &tex_layout);

        let wall_group = wgpu::BindGroupBuilder::new()
            .texture_view(&wall.texture.clone().unwrap())
            .sampler(&wall_sampler)
            .buffer::<MazeUniform>(lights.buffer(), 0..1)
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
        self.lights.upload(
            device,
            encoder,
            MazeUniform {
                light: light.clone(),
                eye: self.camera.desc.eye.to_vec(),
            },
        );
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

struct Pillars {
    floor: Mesh,
    wall: Mesh,
    pillars: Mesh,
    door: Mesh,

    camera: Camera,
    light: PointLight,
    lights: Uniform<MazeUniform>,
    door_offset: f32,

    id_transform_group: wgpu::BindGroup,
    door_transform: Uniform<Matrix4<f32>>,
    door_transform_group: wgpu::BindGroup,

    vertex_group: wgpu::BindGroup,
    floor_group: wgpu::BindGroup,
    wall_group: wgpu::BindGroup,
    pillars_group: wgpu::BindGroup,
    door_group: wgpu::BindGroup,

    depth: wgpu::TextureView,

    pipeline: wgpu::RenderPipeline,
}

impl Pillars {
    fn new(device: &wgpu::Device, window: &Window, encoder: &mut wgpu::CommandEncoder) -> Self {
        let vs_mod = read_shader(device, "pillars.vert.spv");
        let fs_mod = read_shader(device, "pillars.frag.spv");

        let mut objs = read_model("pillars.obj");

        let wall_data = objs.remove(0).meshes.remove(0);
        let pillars_data = objs.remove(0).meshes.remove(0);
        let floor_data = objs.remove(0).meshes.remove(0);
        let door_data = objs.remove(0).meshes.remove(0);

        let floor = Mesh::new(device, window, encoder, &floor_data);
        let wall = Mesh::new(device, window, encoder, &wall_data);
        let pillars = Mesh::new(device, window, encoder, &pillars_data);
        let door = Mesh::new(device, window, encoder, &door_data);

        let camera = Camera::new(
            device,
            CameraDesc {
                eye: (0.0, 2.0, 6.0).into(),
                target: (0.0, 2.0, 40.0).into(),
                up: -Vector3::unit_y(),
                fov: 90.0,
                near: 0.1,
                far: 100.0,
            },
        );

        let light = PointLight {
            pos: Vector4::new(0.0, 4.0, 33.5, 0.0),
            ambient: Vector4::new(0.0, 0.0, 0.0, 0.0),
            diffuse: Vector4::new(1.0, 0.0, 0.0, 0.0),
            specular: Vector4::new(0.0, 0.0, 0.0, 0.0),
            attenuation: Vector4::new(0.55, 0.7, 1.0, 0.0),
        };

        let lights = Uniform::new(device);

        let identity = Uniform::<Matrix4<f32>>::new(device);
        identity.upload(device, encoder, Matrix4::identity());

        let door_transform = Uniform::new(device);

        let transform_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStage::VERTEX, false)
            .build(device);

        let id_transform_group = wgpu::BindGroupBuilder::new()
            .buffer::<Matrix4<f32>>(identity.buffer(), 0..1)
            .build(device, &transform_layout);

        let door_transform_group = wgpu::BindGroupBuilder::new()
            .buffer::<Matrix4<f32>>(door_transform.buffer(), 0..1)
            .build(device, &transform_layout);

        let vertex_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStage::VERTEX, false)
            .build(device);

        let vertex_group = wgpu::BindGroupBuilder::new()
            .buffer::<CameraUniform>(camera.buffer(), 0..1)
            .build(device, &vertex_layout);

        let sampler = wgpu::SamplerBuilder::new()
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
            .sampler(&sampler)
            .buffer::<MazeUniform>(lights.buffer(), 0..1)
            .build(device, &tex_layout);

        let wall_group = wgpu::BindGroupBuilder::new()
            .texture_view(&wall.texture.clone().unwrap())
            .sampler(&sampler)
            .buffer::<MazeUniform>(lights.buffer(), 0..1)
            .build(device, &tex_layout);

        let pillars_group = wgpu::BindGroupBuilder::new()
            .texture_view(&pillars.texture.clone().unwrap())
            .sampler(&sampler)
            .buffer::<MazeUniform>(lights.buffer(), 0..1)
            .build(device, &tex_layout);

        let door_group = wgpu::BindGroupBuilder::new()
            .texture_view(&door.texture.clone().unwrap())
            .sampler(&sampler)
            .buffer::<MazeUniform>(lights.buffer(), 0..1)
            .build(device, &tex_layout);

        let depth = gfx::depth_builder().build(device);
        let depth_view = depth.view().build();

        let pipeline_layout =
            wgpu::create_pipeline_layout(device, &[&vertex_layout, &transform_layout, &tex_layout]);
        let pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
            .fragment_shader(&fs_mod)
            .color_format(gfx::TEXTURE_FORMAT)
            .add_vertex_buffer::<MazeVertex>()
            .depth_format(gfx::DEPTH_FORMAT)
            .build(device);

        Self {
            floor,
            wall,
            pillars,
            door,

            depth: depth_view,

            light,
            lights,
            camera,
            // action
            door_offset: 0.0,

            id_transform_group,
            door_transform,
            door_transform_group,
            vertex_group,
            floor_group,
            wall_group,
            pillars_group,
            door_group,
            pipeline,
        }
    }

    fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        self.camera.update(device, encoder);
        self.lights.upload(
            device,
            encoder,
            MazeUniform {
                light: self.light.clone(),
                eye: self.camera.desc.eye.to_vec(),
            },
        );
        self.door_transform.upload(
            device,
            encoder,
            Matrix4::from_translation(Vector3::new(0.0, self.door_offset * 6.01, 0.0)),
        );
    }

    fn encode(&self, encoder: &mut wgpu::CommandEncoder, texture: &wgpu::TextureView) {
        let mut pass = wgpu::RenderPassBuilder::new()
            .color_attachment(texture, |c| c)
            .depth_stencil_attachment(&self.depth, |d| d)
            .begin(encoder);

        pass.set_pipeline(&self.pipeline);

        pass.set_bind_group(0, &self.vertex_group, &[]);
        pass.set_bind_group(1, &self.id_transform_group, &[]);

        pass.set_bind_group(2, &self.floor_group, &[]);
        pass.set_vertex_buffers(0, &[(&self.floor.buffer, 0)]);
        pass.draw(0..self.floor.len as u32, 0..1);

        pass.set_bind_group(2, &self.wall_group, &[]);
        pass.set_vertex_buffers(0, &[(&self.wall.buffer, 0)]);
        pass.draw(0..self.wall.len as u32, 0..1);

        pass.set_bind_group(2, &self.pillars_group, &[]);
        pass.set_vertex_buffers(0, &[(&self.pillars.buffer, 0)]);
        pass.draw(0..self.pillars.len as u32, 0..1);

        pass.set_bind_group(1, &self.door_transform_group, &[]);
        pass.set_bind_group(2, &self.door_group, &[]);
        pass.set_vertex_buffers(0, &[(&self.door.buffer, 0)]);
        pass.draw(0..self.door.len as u32, 0..1);
    }
}

// Uniform to control the Glitch shader parameters
#[derive(Copy, Clone, Debug)]
struct EffectState {
    t: f32,
    tc: f32,
    pause: f32,
    glitch: f32,
    glitch_mo: f32,
    vhs: f32,
    red: f32,
    flash: f32,
    shake: f32,
    black: f32,
}

// Widget IDs for the tweaking UI
struct Ids {
    constant: widget::Id,
    linear: widget::Id,
    quadratic: widget::Id,
    red: widget::Id,
    green: widget::Id,
    blue: widget::Id,
}

#[derive(Default)]
struct Params {
    t_mul: f32,
    beatstop: bool,
    net: bool,
    red: f32,
    pillars: bool,
    zoom: f32,
}

struct Model {
    audio: Box<dyn Audio>,
    midi: Midi,
    osc: Osc,

    param: Params,
    decay: DecayEnv,
    beat_detect: BeatDetect,
    beat_clock: BeatClock,

    light: PointLight,
    effect_state: EffectState,

    t: f32,
    t_pause: f32,

    font: Font,
    path: Spline<f32, Vector2<f32>>,

    maze: Maze,
    pillars: Pillars,
    drawer: Drawer,
    composite: Composite,
    glitch: Effect<EffectState>,
    present: Present,

    ids: Ids,
    ui: Ui,
    monitor: bool,
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

    let maze = Maze::new(device, &window, &mut encoder);
    let pillars = Pillars::new(device, &window, &mut encoder);
    let glitch = Effect::new(device, "glitch.frag.spv");
    let present = Present::new(device, window.msaa_samples());

    let mut ui = app.new_ui().build().unwrap();
    let ids = Ids {
        constant: ui.generate_widget_id(),
        linear: ui.generate_widget_id(),
        quadratic: ui.generate_widget_id(),
        red: ui.generate_widget_id(),
        green: ui.generate_widget_id(),
        blue: ui.generate_widget_id(),
    };

    // http://wiki.ogre3d.org/tiki-index.php?page=-Point+Light+Attenuation
    let light = PointLight {
        pos: Vector4::new(0.0, 0.0, 0.0, 0.0),
        ambient: Vector4::new(0.0, 0.0, 0.0, 0.0),
        diffuse: Vector4::new(1.0, 1.0, 1.0, 0.0),
        specular: Vector4::new(0.0, 0.0, 0.0, 0.0),
        attenuation: Vector4::new(0.25, 0.1, 1.0, 0.0),
    };

    let path = interp::catmull_loop(
        &vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 6.0),
            Vector2::new(8.0, 6.0),
            Vector2::new(8.0, 10.0),
            Vector2::new(14.0, 10.0),
            Vector2::new(14.0, 16.0),
            Vector2::new(0.0, 16.0),
            Vector2::new(0.0, 6.0),
            Vector2::new(8.0, 6.0),
            Vector2::new(8.0, 10.0),
            Vector2::new(18.0, 10.0),
            Vector2::new(18.0, 0.0),
            Vector2::new(12.0, 0.0),
            Vector2::new(12.0, -6.0),
            Vector2::new(0.0, -6.0),
        ],
        25.0,
    );

    println!("{:?}", path);

    window
        .swap_chain_queue()
        .lock()
        .unwrap()
        .submit(&[encoder.finish()]);

    let state = EffectState {
        t: 0.0,
        tc: 0.0,
        pause: 0.0,
        glitch: 0.0,
        glitch_mo: 0.0,
        vhs: 0.0,
        red: 0.0,
        flash: 0.0,
        shake: 0.0,
        black: 0.0,
    };

    let param = Params::default();

    let decay = DecayEnv::new()
        .with("glitch", 100.0)
        .with("red", 250.0)
        .with("flash", 200.0)
        .with("light", 1000.0);

    Model {
        audio: Box::new(audio::init()),
        midi: Midi::init(),
        osc: Osc::init(34254),

        param,
        decay,
        beat_detect: BeatDetect::new(40.0, 120.0, 0.005, 400.0),
        beat_clock: BeatClock::new(1.0),

        t: 0.0,
        t_pause: 0.0,

        light,
        effect_state: state,

        font: font::from_file("../../resources/fonts/magi.ttf").unwrap(),
        path,

        maze,
        pillars,
        drawer: Drawer::new(device, 4),
        composite: Composite::new(device),
        glitch,
        present,

        ids,
        ui,
        monitor: false,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let ms = update.since_last.as_nanos() as f32 / 1_000_000.0;

    model.audio.update();

    let mut beat_manual = false;

    for (_, message) in model.midi.poll() {
        match message {
            // shader fx
            MidiMessage::Slider(0, f) => model.param.red = f,
            MidiMessage::Slider(1, f) => model.effect_state.glitch = f,
            MidiMessage::Slider(2, f) => model.effect_state.vhs = f,
            MidiMessage::Slider(3, f) => model.effect_state.pause = f,
            MidiMessage::Slider(4, f) => model.effect_state.black = f,

            // Swap maze/pillars

            // Time
            MidiMessage::Knob(6, f) => model.param.t_mul = f,

            // Effect Buttons
            MidiMessage::MainButton(5, b) => model.monitor = b,
            MidiMessage::MainButton(6, true) => model.param.pillars = !model.param.pillars,
            MidiMessage::MainButton(7, true) => beat_manual = true,
            MidiMessage::MainButton(8, true) => model.decay.set("flash"),

            MidiMessage::Slider(6, f) => model.param.zoom = f * 0.3,
            MidiMessage::Slider(7, f) => model.param.zoom = f,
            MidiMessage::Slider(8, f) => model.pillars.door_offset = f,

            // Beat Control
            MidiMessage::CtrlButton(0, true) => model.beat_clock.sync(),
            MidiMessage::CtrlButton(1, true) => model.beat_clock.mul *= 2.0,
            MidiMessage::CtrlButton(2, true) => model.beat_clock.mul /= 2.0,
            MidiMessage::CtrlButton(3, t) => model.param.beatstop = t,
            MidiMessage::CtrlButton(4, t) => model.param.beatstop = t,
            MidiMessage::CtrlButton(5, t) => model.param.net = t,
            MidiMessage::Knob(7, f) => model.beat_detect.bpm_max = 200.0 + f * 300.0,
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

    model.decay.update(ms);

    let t_mod = (10.0 + model.param.t_mul * 75.0) * ms * model.audio.rms();
    model.t += t_mod;
    model.t_pause += t_mod * (1.0 - model.effect_state.pause);

    let audio_beat = model.beat_detect.update(ms, &*model.audio);
    let clock_beat = model.beat_clock.update(ms);
    let beat = if model.param.net {
        clock_beat
    } else {
        audio_beat
    };

    if (beat && !model.param.beatstop && model.effect_state.pause == 0.0) || beat_manual {
        model.decay.set("red");
        model.decay.set("light");
        model.decay.set("glitch");
    }

    model.effect_state.t = model.t;
    model.effect_state.tc = app.time;

    if !model.param.pillars {
        model.effect_state.red = model.param.red * 2.0 * model.decay.v("red");
    }
    model.effect_state.flash = model.decay.v("flash");
    let ambient = Range::new(0.1, 0.0).lerp(model.decay.v("light"));
    model.pillars.light.ambient.x = ambient;
    model.pillars.light.ambient.y = ambient;
    model.pillars.light.ambient.z = ambient;

    model.pillars.light.attenuation.x = Range::new(0.01, 0.0).lerp(model.decay.v("light"));
    model.pillars.light.attenuation.y = 0.0;
    model.pillars.light.attenuation.z = Range::new(3.0, 0.625).lerp(model.decay.v("light"));

    model.pillars.camera.desc.eye.z = 6.0 + 30.0 * model.param.zoom;

    let cam = &mut model.maze.camera.desc;
    let pos_t = model.t_pause / 100.0;
    log::trace!("{} -> {}", pos_t, pos_t % 25.0);
    let pos = model.path.sample(pos_t % 25.0).unwrap();
    let pos_next = model.path.sample((pos_t + 0.1) % 25.0).unwrap();
    cam.eye = Point3::new(pos.x, 0.75, pos.y);
    cam.target = Point3::new(pos_next.x, 0.75, pos_next.y);
    //cam.fov = 90.0 + 5.0 * model.decay.v("red");
    model.light.pos = Vector4::new(pos.x, 0.75, pos.y, 0.0);
    model.light.attenuation.z = 1.0 - model.audio.rms() * 100.0;
    if model.light.attenuation.z < 0.0 {
        model.light.attenuation.z = 0.0
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

    for v in slider(l.attenuation.x, 0.0, 3.0)
        .top_left_with_margin(20.0)
        .label(&format!("quadratic: {}", l.attenuation.x))
        .set(ids.quadratic, ui)
    {
        l.attenuation.x = v;
    }

    for v in slider(l.attenuation.y, 0.0, 1.0)
        .down(10.0)
        .label(&format!("linear: {}", l.attenuation.y))
        .set(ids.linear, ui)
    {
        l.attenuation.y = v;
    }

    for v in slider(l.attenuation.z, 0.0, 5.0)
        .down(10.0)
        .label(&format!("constant: {}", l.attenuation.z))
        .set(ids.constant, ui)
    {
        l.attenuation.z = v;
    }

    /*
    for v in slider(model.pillars.light.attenuation.x, 0.0, 3.0)
        .top_left_with_margin(20.0)
        .label(&format!("quadratic: {}", model.pillars.light.attenuation.x))
        .set(ids.quadratic, ui)
    {
        model.pillars.light.attenuation.x = v;
    }

    for v in slider(model.pillars.light.attenuation.y, 0.0, 1.0)
        .down(10.0)
        .label(&format!("linear: {}", model.pillars.light.attenuation.y))
        .set(ids.linear, ui)
    {
        model.pillars.light.attenuation.y = v;
    }

    */
    /*
    for v in slider(model.pillars.light.attenuation.z, 0.0, 5.0)
        .down(10.0)
        .label(&format!("constant: {}", model.pillars.light.attenuation.z))
        .set(ids.constant, ui)
    {
        model.pillars.light.attenuation.z = v;
    }

    for v in slider(l.diffuse.x, 0.0, 1.0)
        .down(10.0)
        .label(&format!("red: {}", l.diffuse.x))
        .set(ids.red, ui)
    {
        l.diffuse.x = v;
    }

    for v in slider(l.diffuse.y, 0.0, 1.0)
        .down(10.0)
        .label(&format!("green: {}", l.diffuse.y))
        .set(ids.green, ui)
    {
        l.diffuse.y = v;
    }

    for v in slider(l.diffuse.z, 0.0, 1.0)
        .down(10.0)
        .label(&format!("blue: {}", l.diffuse.z))
        .set(ids.blue, ui)
    {
        l.diffuse.z = v;
    }
    */
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
    draw.ellipse()
        .radius(r)
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0);

    // Second layer double ring
    draw.ellipse()
        .radius(r * 0.83)
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0);
    draw.ellipse()
        .radius(r * 0.85)
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0);

    // Outer triangles
    draw.polygon()
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0)
        .points(tri0.iter().map(|p| *p));
    draw.polygon()
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0)
        .points(tri1.iter().map(|p| *p));

    // Triangle point to center lines
    tri0.iter().chain(tri1.iter()).for_each(|p| {
        draw.line().color(WHITE).start(*p).end(pt2(0.0, 0.0));
    });

    // Triangle point rune circles
    tri0.iter()
        .zip(/*"AFH".chars()*/ chars(5.123))
        .for_each(|(p, ch)| {
            draw.ellipse()
                .xy(*p)
                .radius(r * 0.133)
                .color(BLACK)
                .stroke(WHITE)
                .stroke_weight(1.0);
            draw.text(ch)
                .font(font.clone())
                .font_size((r * 0.12).round() as u32)
                .xy(*p)
                .color(WHITE);
        });

    // Inner circle ring
    draw.ellipse()
        .radius(r * 0.5)
        .color(BLACK)
        .stroke(WHITE)
        .stroke_weight(1.0);

    // Inner runes
    poly_pts(26, t / 500.0)
        .map(|p| p * r * 0.465)
        .zip(lib::chars(1.0))
        .enumerate()
        .for_each(|(i, (p, ch))| {
            let angle = i as f32 / 30.0 * -TAU;
            draw.text(ch)
                .font(font.clone())
                .font_size((r * 0.04).round() as u32)
                .xy(p)
                .rotate(angle)
                .color(WHITE);
        });

    // Innner circle fill
    draw.ellipse()
        .radius(r * 0.433)
        .color(BLACK)
        .stroke(WHITE)
        .stroke_weight(1.0);

    // Inner tri
    draw.polygon()
        .color(BLACK)
        .stroke(WHITE)
        .stroke_weight(1.0)
        .points(tri0.iter().map(|p| *p * 0.5));

    tri0.iter().for_each(|p| {
        // Outer fat circles
        draw.ellipse()
            .xy(*p * 0.7)
            .radius(r * 0.033)
            .color(BLACK)
            .stroke(WHITE)
            .stroke_weight(2.0);

        // Inner fat circle lines
        draw.line().start(*p / 2.0).end(pt2(0.0, 0.0)).color(WHITE);

        // Inner fat circles
        draw.ellipse()
            .xy(*p * 0.3)
            .radius(r * 0.033)
            .color(BLACK)
            .stroke(WHITE)
            .stroke_weight(4.0);
    });

    // Center outer, smaller, mid, point
    draw.ellipse()
        .color(BLACK)
        .stroke(WHITE)
        .stroke_weight(1.0)
        .radius(r * 0.2);
    draw.ellipse()
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0)
        .radius(r * 0.1733);
    draw.ellipse()
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0)
        .radius(r * 0.1);
    draw.ellipse().color(WHITE).radius(r * 0.033);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let tframe = Instant::now();

    // Scope our draw code so we can clean it up and later render the UI, which eeds to mutably
    // borrow the Frame as well
    {
        let draw = app.draw();
        draw.background().color(BLACK);
        if !model.param.pillars {
            demon1(
                &draw,
                model.font.clone(),
                &pt2(0.0, 0.0),
                200.0 + 100.0 * model.decay.v("red"),
                model.t_pause,
            );
        }

        let window = app.main_window();
        let device = window.swap_chain_device();
        let mut encoder = frame.command_encoder();

        let before = Instant::now();
        if model.param.pillars {
            model.pillars.update(device, &mut encoder);
            model.pillars.encode(&mut encoder, &model.composite.view1);
        } else {
            model.maze.update(device, &mut encoder, &model.light);
            model.maze.encode(&mut encoder, &model.composite.view1);
        }
        log::trace!("Geometry in {:?}", before.elapsed());

        let before = Instant::now();
        model
            .drawer
            .encode(device, &mut encoder, &model.composite.view2, &draw);
        log::trace!("Overdraw in {:?}", before.elapsed());

        let before = Instant::now();
        model.composite.encode(&mut encoder, model.glitch.view());
        model
            .glitch
            .update(device, &mut encoder, &model.effect_state);
        model.glitch.encode(&mut encoder, model.present.view());
        log::trace!("Shaders in {:?}", before.elapsed());

        let before = Instant::now();
        model.present.encode(&mut encoder, &frame);
        log::trace!("Present in {:?}", before.elapsed());
    }

    if model.monitor {
        let before = Instant::now();
        model.ui.draw_to_frame(app, &frame).unwrap();
        log::trace!("UI in {:?}", before.elapsed());
    }

    let eframe = tframe.elapsed();
    let seconds = eframe.as_nanos() as f32 / 1_000_000.0;
    log::trace!("Frame in {:?} / {}", tframe.elapsed(), 1.0 / seconds);
}
