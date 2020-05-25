use nannou::wgpu;

use super::camera::{Camera, CameraDesc, CameraUniform};
use super::lights::{Lights, LightsInfoUniform, PointLight, PointLightUniform};
use super::material::MaterialUniform;
use super::model::{Model, TransformUniform, VertexDescriptor};

use crate as lib;

pub struct Scene {
    pub models: Vec<Model>,
    pub lights: Lights,
    pub camera: Camera,
    // action
    depth: wgpu::TextureView,
    lights_group: wgpu::BindGroup,
    camera_group: wgpu::BindGroup,
    model_groups: Vec<(wgpu::BindGroup, Vec<wgpu::BindGroup>)>,
    pub pipeline: wgpu::RenderPipeline,
}

impl Scene {
    pub fn new(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        models: Vec<Model>,
        cam: CameraDesc,
        points: Vec<PointLight>,
    ) -> Self {
        let lights = Lights::new(device, encoder, points);
        let camera = Camera::new(device, cam);

        let depth = super::depth_builder().build(device).view().build();

        let vs_mod = lib::read_shader(device, "scene.vert.spv");
        let fs_mod = lib::read_shader(device, "scene.frag.spv");

        let lights_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStage::FRAGMENT, false)
            .uniform_buffer(wgpu::ShaderStage::FRAGMENT, false)
            .build(device);

        let lights_group = wgpu::BindGroupBuilder::new()
            .buffer::<LightsInfoUniform>(&lights.info.buffer, 0..1)
            .buffer::<PointLightUniform>(&lights.points_buffer, 0..Lights::MAX_POINT)
            .build(device, &lights_layout);

        let camera_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStage::VERTEX, false)
            .build(device);

        let camera_group = wgpu::BindGroupBuilder::new()
            .buffer::<CameraUniform>(&camera.uniform.buffer, 0..1)
            .build(device, &camera_layout);

        let model_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStage::VERTEX, false)
            .build(device);

        let dim = wgpu::TextureViewDimension::D2;
        let object_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStage::VERTEX, false)
            .uniform_buffer(wgpu::ShaderStage::FRAGMENT, false)
            .sampler(wgpu::ShaderStage::FRAGMENT)
            .sampled_texture(wgpu::ShaderStage::FRAGMENT, false, dim)
            .build(device);

        let model_groups = models
            .iter()
            .map(|m| {
                let object_groups = m
                    .objects
                    .iter()
                    .map(|o| {
                        let sampler = wgpu::SamplerBuilder::new()
                            .mag_filter(o.material.filter)
                            .address_mode(wgpu::AddressMode::Repeat)
                            .build(&device);

                        let group = wgpu::BindGroupBuilder::new()
                            .buffer::<TransformUniform>(&o.transform.uniform.buffer, 0..1)
                            .buffer::<MaterialUniform>(&o.material.uniform.buffer, 0..1)
                            .sampler(&sampler)
                            .texture_view(o.material.diffuse.as_ref().unwrap())
                            .build(device, &object_layout);

                        group
                    })
                    .collect();

                let model_group = wgpu::BindGroupBuilder::new()
                    .buffer::<TransformUniform>(&m.transform.uniform.buffer, 0..1)
                    .build(device, &model_layout);

                (model_group, object_groups)
            })
            .collect();

        let pipeline_layout =
            wgpu::create_pipeline_layout(device, &[&lights_layout, &camera_layout, &model_layout, &object_layout]);

        let pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
            .fragment_shader(&fs_mod)
            .color_format(super::FORMAT)
            .add_vertex_buffer::<VertexDescriptor>()
            .depth_format(super::DEPTH_FORMAT)
            .build(device);

        Self {
            models,
            lights,
            camera,

            depth,
            lights_group,
            camera_group,
            model_groups,
            pipeline,
        }
    }

    pub fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        self.lights.update(device, encoder);

        self.camera.update(device, encoder);

        for m in &self.models {
            m.update(device, encoder);
        }
    }

    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, texture: &wgpu::TextureView) {
        let mut pass = wgpu::RenderPassBuilder::new()
            .color_attachment(texture, |c| c)
            .depth_stencil_attachment(&self.depth, |d| d)
            .begin(encoder);

        pass.set_pipeline(&self.pipeline);

        pass.set_bind_group(0, &self.lights_group, &[]);
        pass.set_bind_group(1, &self.camera_group, &[]);

        for (m, (model_group, object_groups)) in self.models.iter().zip(self.model_groups.iter()) {
            pass.set_bind_group(2, &model_group, &[]);

            for (o, group) in m.objects.iter().zip(object_groups) {
                pass.set_bind_group(3, &group, &[]);
                pass.set_vertex_buffers(0, &[(&o.mesh.buffer, 0)]);
                pass.draw(0..o.mesh.verts.len() as u32, 0..1);
            }
        }
    }
}
