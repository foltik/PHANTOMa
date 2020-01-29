use rendy::{
    graph::render::RenderPassNodeBuilder,
    hal::{Backend},
    shader::{ShaderKind, SourceLanguage, SourceShaderInfo, SpirvShader},
};

pub mod triangle;

pub trait Component<B: Backend, T: Sized>: 'static + Sized {
    //fn resize(&mut self);
    //fn update(&mut self, event: WindowEvent);
    //fn render(&self);
    fn builder() -> RenderPassNodeBuilder<B, T>;
}

pub fn load_shader(data: &'static str, kind: ShaderKind) -> SpirvShader {
    SourceShaderInfo::new(data, "", kind, SourceLanguage::GLSL, "main")
        .precompile()
        .unwrap()
}
