/*
use rendy::{
    factory::Factory,
    hal::{device::Device, pso, Backend},
    resource::{DescriptorSet as RDescriptorSet, Escape, Handle},
};
use std::collections::BTreeSet;
use enum_dispatch::enum_dispatch;

pub use pso::DescriptorBinding;
pub use pso::ShaderStageFlags;

pub mod buffer;
pub mod sampler;

use buffer::{Uniform, UniformInfo};
use sampler::{Sampler, SamplerInfo};

pub enum DescriptorType {
    Uniform(Box<dyn UniformInfo>),
    Sampler(SamplerInfo)
}

pub struct DescriptorSet<B: Backend> {
    descriptors: Vec<Box<dyn Descriptor<B>>>,

    set: Option<Escape<RDescriptorSet<B>>>,
}

impl<B: Backend> DescriptorSet<B> {
    pub fn new(descriptors: Vec<Box<dyn Descriptor<B>>>) -> Self {
        let mut uniq = BTreeSet::new();
        let dup = descriptors.iter().all(|d| uniq.insert(d.binding()));
        assert_eq!(dup, false, "Layout: Duplicate binding!");

        Self {
            descriptors,
            set: None,
        }
    }

    pub fn alloc(&mut self, factory: &Factory<B>) {
        let layout_bindings = self
            .descriptors
            .iter()
            .map(|d| d.set_layout())
            .collect::<Vec<_>>();

        let set_layout = Handle::from(
            factory
                .create_descriptor_set_layout(layout_bindings)
                .unwrap(),
        );
        let set = factory.create_descriptor_set(set_layout).unwrap();

        self.descriptors.iter_mut().for_each(|d| d.alloc(factory));
        let writes = self
            .descriptors
            .iter()
            .map(|d| d.set_write(set.raw()))
            .collect::<Vec<_>>();

        unsafe {
            factory.write_descriptor_sets(writes);
        }

        self.set = Some(set);
    }
}
*/
