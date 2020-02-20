use rendy::shader::{SourceLanguage, SourceShaderInfo};

pub use rendy::shader::{ShaderKind, ShaderSetBuilder};

pub type Shader = rendy::shader::SpirvShader;

pub fn from_source(source: &'static str, kind: ShaderKind) -> Shader {
    SourceShaderInfo::new(source, "", kind, SourceLanguage::GLSL, "main")
        .precompile()
        .expect("Failed to precompile shader!")
}
