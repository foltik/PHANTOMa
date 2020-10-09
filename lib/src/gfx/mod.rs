pub mod wgpu;

pub mod frame;
pub mod uniform;
pub mod pass;
// pub mod std140;

// pub mod camera;
// pub mod lights;
// pub mod material;
// pub mod mesh;
// pub mod model;
// pub mod scene;

// use crate as lib;

// use std::cell::RefCell;
// use std::fmt::Debug;

// pub use camera::{Camera, CameraDesc, CameraUniform};
// pub use mesh::Mesh;

// // TODO: put this shit in multiple files

// pub const RESOLUTION: [u32; 2] = [1920, 1080];
// pub const ASPECT: f32 = 1920.0 / 1080.0;
// pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Unorm;
// pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

// pub fn texture_builder() -> wgpu::TextureBuilder {
//     wgpu::TextureBuilder::new().size(RESOLUTION).format(FORMAT)
// }

// pub fn depth_builder() -> wgpu::TextureBuilder {
//     wgpu::TextureBuilder::new()
//         .size(RESOLUTION)
//         .format(DEPTH_FORMAT)
//         .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT)
// }


// // TODO: this is mostly an Effect with two input images and a hard coded
// // shader that sums the two image values.
// // I only made it because for some reason a Draw clears out the image from Maze
// // before drawing, so either figure out how to genericize Effect over N input
// // images, or fix Draw clearing the image.
// pub struct Composite {
//     pub view1: wgpu::TextureView,
//     pub view2: wgpu::TextureView,
//     bind_group: wgpu::BindGroup,
//     pipeline: wgpu::RenderPipeline,
// }

// impl Composite {
//     pub fn new(device: &wgpu::Device) -> Self {
//         let vs_mod = lib::read_shader(device, BILLBOARD_SHADER);
//         let fs_mod = lib::read_shader(device, "add.frag.spv");

//         let tex1 = texture_builder()
//             .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
//             .build(device);
//         let view1 = tex1.view().build();

//         let tex2 = texture_builder()
//             .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
//             .build(device);
//         let view2 = tex2.view().build();

//         let sampler = wgpu::SamplerBuilder::new().build(&device);

//         let bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
//             .sampled_texture_from(wgpu::ShaderStage::FRAGMENT, &tex1)
//             .sampled_texture_from(wgpu::ShaderStage::FRAGMENT, &tex2)
//             .sampler(wgpu::ShaderStage::FRAGMENT)
//             .build(device);

//         let bind_group = wgpu::BindGroupBuilder::new()
//             .texture_view(&view1)
//             .texture_view(&view2)
//             .sampler(&sampler)
//             .build(device, &bind_group_layout);

//         let pipeline_layout = wgpu::create_pipeline_layout(device, &[&bind_group_layout]);
//         let pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
//             .fragment_shader(&fs_mod)
//             .color_format(FORMAT)
//             .build(device);

//         Self {
//             view1,
//             view2,
//             bind_group,
//             pipeline,
//         }
//     }

//     pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
//         let mut pass = wgpu::RenderPassBuilder::new()
//             .color_attachment(target, |c| c)
//             .begin(encoder);

//         pass.set_pipeline(&self.pipeline);
//         pass.set_bind_group(0, &self.bind_group, &[]);

//         pass.draw(0..3, 0..1);
//     }
// }

// pub struct Present {
//     effect: Effect<()>,
// }

// impl Present {
//     pub fn new(device: &wgpu::Device, samples: u32) -> Self {
//         Self {
//             effect: Effect::new_internal(device, samples, PASSTHROUGH_SHADER),
//         }
//     }

//     pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, frame: &nannou::Frame) {
//         self.effect.encode(encoder, frame.texture_view())
//     }

//     pub fn view(&self) -> &wgpu::TextureView {
//         self.effect.view()
//     }
// }

// pub struct Drawer {
//     renderer: RefCell<nannou::draw::Renderer>,
//     reshaper: wgpu::TextureReshaper,
//     texture: wgpu::Texture,
// }

// impl Drawer {
//     pub fn new(device: &wgpu::Device, samples: u32) -> Self {
//         let texture = texture_builder()
//             .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
//             .sample_count(samples)
//             .build(device);

//         let texture_view = texture.view().build();

//         let renderer = nannou::draw::RendererBuilder::new()
//             .build_from_texture_descriptor(device, texture.descriptor());

//         let reshaper = wgpu::TextureReshaper::new(device, &texture_view, samples, 1, FORMAT);

//         Self {
//             renderer: RefCell::new(renderer),
//             reshaper,
//             texture,
//         }
//     }

//     pub fn encode(
//         &self,
//         device: &wgpu::Device,
//         encoder: &mut wgpu::CommandEncoder,
//         target: &wgpu::TextureView,
//         draw: &nannou::Draw,
//     ) {
//         self.renderer
//             .borrow_mut()
//             .render_to_texture(device, encoder, draw, &self.texture);

//         self.reshaper.encode_render_pass(target, encoder);
//     }
// }