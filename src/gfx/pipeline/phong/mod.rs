use crate::app::App;

use crate::gfx::frame::Frame;
use crate::gfx::mesh::{Vertex, VertexExt as _};
use crate::gfx::scene::Scene;
use crate::gfx::wgpu;

mod material;
use material::Material;

pub struct PhongPipeline {
    pub layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,

    pub mats: Vec<Material>,
    pub meshes: Vec<Vec<usize>>,

    pipeline: wgpu::RenderPipeline,
}

impl PhongPipeline {
    pub fn new(app: &App, scene: &Scene) -> Self {
        Self::new_filtered(app, scene, |_| true, |_| true)
    }

    pub fn new_filtered<NodeFn, MatFn>(app: &App, scene: &Scene, mut node_filter: NodeFn, mut mat_filter: MatFn) -> Self
    where
        NodeFn: FnMut(&str) -> bool,
        MatFn: FnMut(&str) -> bool,
    {
        let vs = crate::resource::read_shader(&app.device, "phong.vert.spv");
        let fs = crate::resource::read_shader(&app.device, "phong.frag.spv");

        let layout = wgpu::util::BindGroupLayoutBuilder::new("phong")
            .tex(wgpu::ShaderStages::FRAGMENT)
            .sampler(wgpu::ShaderStages::FRAGMENT)
            .uniform(wgpu::ShaderStages::FRAGMENT)
            .build(&app.device);

        let sampler = wgpu::util::SamplerBuilder::new("phong")
            .address_mode(wgpu::AddressMode::Repeat)
            .mag_filter(wgpu::FilterMode::Nearest)
            .build(&app.device);

        let pipeline = wgpu::util::PipelineBuilder::new("phong")
            .with_layout(&scene.cam_layout)
            .with_layout(&scene.light_layout)
            .with_layout(&layout)
            .with_layout(&scene.mesh_layout)
            .render(&vs)
            .fragment(&fs)
            .add_vertex_buffer::<Vertex>(Vertex::ty().attrs())
            .hack_add_default_depth_stencil_state()
            .build(&app.device);

        let mats = scene
            .desc
            .materials
            .iter()
            .filter(|mat| mat_filter(&mat.name))
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
                        m.material == i && node_filter(&scene.desc.nodes[scene.mesh_idxs[*j]].name)
                    })
                    .map(|(j, _)| j)
                    .collect()
            })
            .collect();

        Self {
            layout,
            sampler,
            pipeline,

            mats,
            meshes,
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
            .color_attachment(target, |c| c)
            .depth_stencil_attachment(depth, |d| d)
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

    pub fn encode_load(
        &self,
        frame: &mut Frame,
        scene: &Scene,
        depth: &wgpu::RawTextureView,
        target: &wgpu::RawTextureView,
    ) {
        let mut pass = wgpu::util::RenderPassBuilder::new()
            .color_attachment(target, |c| c.color(|o| o.load()))
            .depth_stencil_attachment(depth, |d| d)
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
