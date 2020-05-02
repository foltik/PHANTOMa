/*
pub mod descriptor;

use descriptor::{
    DescriptorType,
    buffer::Uniform,
    sampler::Sampler,
};

use rendy::hal::Backend;
use std::any::Any;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Layout<B: Backend> {
    samplers: HashMap<String, Sampler<B>>,
    uniforms: HashMap<String, Box<dyn Uniform<B>>>,
}

impl<B: Backend> Layout<B> {
    fn with_descriptors(descriptors: HashMap<String, DescriptorType>) -> Self {
        let mut inst: Layout<B> = Default::default();

        descriptors.for_each(|(k,d)| match d {
            DescriptorType::Sampler(i) => inst.samplers.insert(k, Sampler::new(i)),
            DescriptorType::Uniform(i) => inst.uniforms.insert(k, i.new()),
            _ => panic!("Layout: Unknown DescriptorType!"),
        });

        inst
    }

    fn from_reflect() /* -> Self */ {}

    fn attr<S: Into<String>>(attr: S) {}
    fn bind() {}
}

macro_rules! layout {
    (struct $name:ident { $($field:ident : $ty:ty),* }) => {};
}
*/

// Ideal
/*
layout!{
    struct MyLayout {
        data: Uniform,
        layout:
        prev: Uniform,

    }
}
*/
