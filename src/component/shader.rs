use rendy::{
    hal::{Backend, pso}
};

pub fn shader_set<'a, B: Backend>(
    vertex: &'a B::ShaderModule,
    fragment: Option<&'a B::ShaderModule>,
) -> pso::GraphicsShaderSet<'a, B> {
    shader_set_ext(vertex, fragment, None, None, None)
}

#[allow(dead_code)]
pub fn shader_set_ext<'a, B: Backend>(
    vertex: &'a B::ShaderModule,
    fragment: Option<&'a B::ShaderModule>,
    hull: Option<&'a B::ShaderModule>,
    domain: Option<&'a B::ShaderModule>,
    geometry: Option<&'a B::ShaderModule>,
) -> pso::GraphicsShaderSet<'a, B> {
    fn map_entry_point<B: Backend>(module: &B::ShaderModule) -> pso::EntryPoint<'_, B> {
        pso::EntryPoint {
            entry: "main",
            module,
            specialization: pso::Specialization::default(),
        }
    }

    pso::GraphicsShaderSet {
        vertex: map_entry_point(vertex),
        fragment: fragment.map(map_entry_point),
        hull: hull.map(map_entry_point),
        domain: domain.map(map_entry_point),
        geometry: geometry.map(map_entry_point),
    }
}
