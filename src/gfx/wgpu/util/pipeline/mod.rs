use crate::gfx::wgpu::{BindGroupLayout, PushConstantRange, ShaderModule};

mod render_pipeline;
use render_pipeline::RenderPipelineBuilder;

// mod compute_pipeline;
// use compute_pipeline::ComputePipelineBuilder;

#[derive(Debug)]
pub struct PipelineBuilder<'a> {
    label: &'a str,
    layouts: Vec<&'a BindGroupLayout>,
    constants: Vec<PushConstantRange>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            layouts: vec![],
            constants: vec![]
        }
    }

    pub fn with_layout(mut self, layout: &'a BindGroupLayout) -> Self {
        self.layouts.push(layout);
        self
    }

    pub fn with_constants(mut self, constant: PushConstantRange) -> Self {
        self.constants.push(constant);
        self
    }

    pub fn render(self, vs_mod: &'a ShaderModule) -> RenderPipelineBuilder<'a> {
        RenderPipelineBuilder::new(self, vs_mod)
    }

    // pub fn compute(self) -> ComputePipelineBuilder<'a> {

    // }
}
