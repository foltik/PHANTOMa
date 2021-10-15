use std::collections::HashMap;

use lib::app::App;

use lib::gfx::frame::Frame;
use lib::gfx::mesh::{Vertex, VertexExt as _};
use lib::gfx::scene::Scene;
use lib::gfx::wgpu;

mod material;
pub use material::{Material, MaterialDesc};

pub struct Animated {
    pub layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,

    pub mats: Vec<Material>,
    pub mats_named: HashMap<String, usize>,
    pub meshes: Vec<Vec<usize>>,

    pipeline: wgpu::RenderPipeline,
}

impl Animated {
    pub fn new(app: &App, scene: &Scene, descs: &[MaterialDesc]) -> Self {
        let vs = lib::resource::read_shader(&app.device, "phong.vert.spv");
        let fs = lib::resource::read_shader(&app.device, "phong_anim.frag.spv");

        let layout = wgpu::util::BindGroupLayoutBuilder::new("phong")
            .array_tex(wgpu::ShaderStage::FRAGMENT)
            .sampler(wgpu::ShaderStage::FRAGMENT)
            .uniform(wgpu::ShaderStage::FRAGMENT)
            .build(&app.device);

        let sampler = wgpu::util::SamplerBuilder::new("phong")
            .address_mode(wgpu::AddressMode::Repeat)
            .mag_filter(wgpu::FilterMode::Nearest)
            .min_filter(wgpu::FilterMode::Nearest)
            .build(&app.device);

        let pipeline = wgpu::util::PipelineBuilder::new("phong")
            .with_layout(&scene.cam_layout)
            .with_layout(&scene.light_layout)
            .with_layout(&layout)
            .with_layout(&scene.mesh_layout)
            .render(&vs)
            .fragment_shader(&fs)
            .add_vertex_buffer::<Vertex>(Vertex::ty().attrs())
            .index_format(wgpu::IndexFormat::Uint32)
            .depth_stencil()
            .build(&app.device);

            log::debug!("loading mats");

        let mats = descs
            .iter()
            .map(|desc| {
                let imgs = desc
                    .images
                    .iter()
                    .map(|i| lib::resource::read_image(i))
                    .collect::<Vec<_>>();

                let (image, scale) = wgpu::util::image::load_array(app, &imgs);

                log::debug!("done load array");

                let view = image
                    .view()
                    .dimension(wgpu::TextureViewDimension::D2Array)
                    .array_layer_count(imgs.len() as u32)
                    .build();

                Material::new(app, &desc, scale, &layout, &sampler, &view)
            })
            .collect::<Vec<_>>();

            log::debug!("done mats");

        let mut mats_named = HashMap::new();
        for (i, mat) in mats.iter().enumerate() {
            mats_named.insert(mat.name.clone(), i);
        }

        // TODO: This is wildly inefficient
        let mut meshes = vec![vec![]; mats.len()];
        for (i, &nidx) in scene.mesh_idxs.iter().enumerate() {
            let name = &scene.desc.nodes[nidx].name;
            for d in descs {
                if d.nodes.iter().find(|n| *n == name).is_some() {
                    meshes[mats_named[d.name]].push(i);
                }
            }
        }

        for (i, mesh) in scene.desc.meshes.iter().enumerate() {
            let name = &scene.desc.materials[mesh.material].name;
            for d in descs {
                if d.mats.iter().find(|n| *n == name).is_some() {
                    meshes[mats_named[d.name]].push(i);
                }
            }
        }

        log::debug!("done animated");

        Self {
            layout,
            sampler,
            pipeline,

            mats,
            mats_named,
            meshes,
        }
    }

    pub fn update(&mut self, t: f32) {
        for mat in &mut self.mats {
            let i = (t * mat.fps as f32).floor() as u32 % mat.fps;
            mat.uniform.i = i;
        }
    }

    pub fn upload(&self, frame: &mut Frame) {
        for mat in &self.mats {
            mat.uniform.upload(frame);
        }
    }

    pub fn encode(
        &self,
        frame: &mut Frame,
        scene: &Scene,
        depth: &wgpu::RawTextureView,
        target: &wgpu::RawTextureView,
    ) {
        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(target, |c| c.color(|o| o.load()))
            .depth_stencil_attachment(depth, |d| d.depth(|o| o.load()))
            .begin(frame);

        pass.set_pipeline(&self.pipeline);

        scene.cam.bind(&mut pass, 0);
        scene.lights.bind(&mut pass, 1);

        for (mat, meshes) in self.mats.iter().zip(self.meshes.iter()) {
            mat.bind(&mut pass, 2);

            for i in meshes {
                scene.meshes[*i].draw(&mut pass, 3);
            }
        }
    }
}
