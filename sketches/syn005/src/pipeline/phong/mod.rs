use lib::app::App;

use lib::gfx::frame::Frame;
use lib::gfx::mesh::{Vertex, VertexExt as _};
use lib::gfx::scene::Scene;
use lib::gfx::wgpu;

mod material;
use material::Material;

pub struct Phong {
    pub layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,

    pub mats: Vec<Material>,
    pub meshes: Vec<Vec<usize>>,

    depth: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
}

impl Phong {
    pub fn new<F>(app: &App, scene: &Scene, mut f: F) -> Self
    where
        F: FnMut(&str) -> bool,
    {
        let vs = lib::resource::read_shader(&app.device, "phong.vert.spv");
        let fs = lib::resource::read_shader(&app.device, "phong.frag.spv");

        let layout = wgpu::util::BindGroupLayoutBuilder::new("phong")
            .tex(wgpu::ShaderStage::FRAGMENT)
            .sampler(wgpu::ShaderStage::FRAGMENT)
            .uniform(wgpu::ShaderStage::FRAGMENT)
            .build(&app.device);

        let sampler = wgpu::util::SamplerBuilder::new("phong")
            .address_mode(wgpu::AddressMode::Repeat)
            .mag_filter(wgpu::FilterMode::Nearest)
            .build(&app.device);

        let depth = wgpu::util::TextureBuilder::new_depth("depth")
            .build(&app.device)
            .view()
            .build();

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

        let mats = scene
            .desc
            .materials
            .iter()
            .map(|mat| Material::new(app, &layout, &sampler, mat))
            .collect();

        let meshes = scene
            .desc
            .materials
            .iter()
            .enumerate()
            .map(|(i, _)| {
                scene
                    .desc
                    .meshes
                    .iter()
                    .enumerate()
                    .filter(|(j, m)| {
                        m.material == i && f(&scene.desc.nodes[scene.mesh_idxs[*j]].name)
                    })
                    .map(|(j, _)| j)
                    .collect()
            })
            .collect();


        Self {
            layout,
            sampler,
            depth,
            pipeline,

            mats,
            meshes,
        }
    }

    pub fn encode(&self, frame: &mut Frame, scene: &Scene, target: &wgpu::RawTextureView) {
        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(target, |c| c)
            .depth_stencil_attachment(&self.depth, |d| d)
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
