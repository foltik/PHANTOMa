use nannou::math::cgmath::{self, Array, Vector3};
use nannou::wgpu;

use super::camera::{Camera, CameraDesc, CameraMetaUniform, CameraUniform};
use super::lights::{Lights, LightsInfoUniform, PointLight, PointLightUniform};
use super::material::MaterialUniform;
use super::model::{Model, TransformUniform, VertexDescriptor};
use super::{Composite, Effect};

use crate as lib;

pub struct SceneParam {
    pub bloom: f32,
}

pub struct Scene {
    pub models: Vec<Model>,
    pub lights: Lights,
    pub camera: Camera,

    pub do_bloom: bool,

    lights_group: wgpu::BindGroup,
    camera_group: wgpu::BindGroup,
    model_groups: Vec<(wgpu::BindGroup, Vec<wgpu::BindGroup>)>,

    phong_pipeline: wgpu::RenderPipeline,
    phong_depth: wgpu::TextureView,
    emit_pipeline: wgpu::RenderPipeline,
    emit_depth: wgpu::TextureView,

    blur1: Effect<f32>,
    blur2: Effect<f32>,
    composite: Composite,
}

impl Scene {
    pub fn new(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        models: Vec<Model>,
        cam: CameraDesc,
        ambient: f32,
        points: Vec<PointLight>,
    ) -> Self {
        let lights = Lights::new(device, encoder, ambient, points);
        let camera = Camera::new(device, cam);

        let lights_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStage::FRAGMENT, false)
            .uniform_buffer(wgpu::ShaderStage::FRAGMENT, false)
            .build(device);

        let lights_group = wgpu::BindGroupBuilder::new()
            .buffer::<LightsInfoUniform>(&lights.info_uniform.buffer, 0..1)
            .buffer::<PointLightUniform>(&lights.points_uniform.buffer, 0..Lights::MAX_POINT)
            .build(device, &lights_layout);

        let camera_layout = wgpu::BindGroupLayoutBuilder::new()
            .uniform_buffer(wgpu::ShaderStage::VERTEX, false)
            .uniform_buffer(wgpu::ShaderStage::FRAGMENT, false)
            .build(device);

        let camera_group = wgpu::BindGroupBuilder::new()
            .buffer::<CameraUniform>(&camera.transform.buffer, 0..1)
            .buffer::<CameraMetaUniform>(&camera.meta.buffer, 0..1)
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

        let do_bloom = models.iter().any(|m| {
            m.objects
                .iter()
                .any(|o| cgmath::dot(o.material.desc.emissive.col, Vector3::from_value(1.0)) > 0.0)
        });

        let pipeline_layout = wgpu::create_pipeline_layout(
            device,
            &[
                &lights_layout,
                &camera_layout,
                &model_layout,
                &object_layout,
            ],
        );

        let vs_mod = lib::read_shader(device, "scene.vert.spv");
        let phong_fs_mod = lib::read_shader(device, "scene_phong.frag.spv");
        let emit_fs_mod = lib::read_shader(device, "scene_emit.frag.spv");

        let phong_pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
            .fragment_shader(&phong_fs_mod)
            .color_format(super::FORMAT)
            .add_vertex_buffer::<VertexDescriptor>()
            .depth_format(super::DEPTH_FORMAT)
            .build(device);

        let emit_pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
            .fragment_shader(&emit_fs_mod)
            .color_format(super::FORMAT)
            .add_vertex_buffer::<VertexDescriptor>()
            .depth_format(super::DEPTH_FORMAT)
            .build(device);

        let phong_depth = super::depth_builder().build(device).view().build();
        let emit_depth = super::depth_builder().build(device).view().build();

        // TODO: Just use two images and instead of duplicating shaders
        let blur1 = Effect::new(device, "scene_blur_h.frag.spv");
        let blur2 = Effect::new(device, "scene_blur_v.frag.spv");
        let composite = Composite::new(device);

        Self {
            models,
            lights,
            camera,

            do_bloom,

            lights_group,
            camera_group,
            model_groups,

            phong_pipeline,
            phong_depth,
            emit_pipeline,
            emit_depth,

            blur1,
            blur2,
            composite,
        }
    }

    pub fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        self.lights.update(device, encoder);
        self.camera.update(device, encoder);

        for m in &self.models {
            m.update(device, encoder);
        }
    }

    fn encode_to_pass(&self, pass: &mut wgpu::RenderPass) {
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

    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, texture: &wgpu::TextureView) {
        {
            let target = if self.do_bloom {
                &self.composite.view1
            } else {
                texture
            };

            let mut phong_pass = wgpu::RenderPassBuilder::new()
                .color_attachment(target, |c| c)
                .depth_stencil_attachment(&self.phong_depth, |d| d)
                .begin(encoder);

            phong_pass.set_pipeline(&self.phong_pipeline);
            self.encode_to_pass(&mut phong_pass);
        }

        if self.do_bloom {
            {
                let mut emit_pass = wgpu::RenderPassBuilder::new()
                    .color_attachment(&self.blur1.view, |c| c)
                    .depth_stencil_attachment(&self.emit_depth, |d| d)
                    .begin(encoder);

                emit_pass.set_pipeline(&self.emit_pipeline);
                self.encode_to_pass(&mut emit_pass);
            }

            for _ in 0..8 {
                self.blur1.encode(encoder, &self.blur2.view);
                self.blur2.encode(encoder, &self.blur1.view);
            }

            self.blur1.encode(encoder, &self.composite.view2);

            self.composite.encode(encoder, texture);
        }
    }
}
