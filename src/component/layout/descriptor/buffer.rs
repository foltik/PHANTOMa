use super::{Descriptor, DescriptorBinding, ShaderStageFlags};
use glsl_layout::AsStd140;
use rendy::{
    factory::Factory,
    hal::{self, pso, Backend},
    resource::{Buffer, BufferInfo, Escape},
};
use std::marker::PhantomData;

pub struct Uniform<B: Backend, T: AsStd140>
where
    T::Std140: Sized,
{
    pub binding: DescriptorBinding,
    pub stage: pso::ShaderStageFlags,
    marker: PhantomData<T>,

    buffer: Option<Escape<Buffer<B>>>,
}

impl<B: Backend, T: AsStd140> Uniform<B, T>
where
    T::Std140: Sized,
{
    pub fn new(binding: DescriptorBinding, stage: ShaderStageFlags) -> Self {
        Self {
            binding,
            stage,
            marker: PhantomData,
            buffer: None,
        }
    }

    fn size() -> usize {
        std::mem::size_of::<<T as AsStd140>::Std140>()
    }
}

impl<B: Backend, T: AsStd140> Descriptor<B> for Uniform<B, T>
where
    T::Std140: Sized,
{
    fn binding(&self) -> DescriptorBinding {
        self.binding
    }

    fn alloc(&mut self, factory: &Factory<B>) {
        let buffer = factory
            .create_buffer(
                BufferInfo {
                    size: Self::size() as u64,
                    usage: hal::buffer::Usage::UNIFORM,
                },
                rendy::memory::Dynamic,
            )
            .unwrap();

        self.buffer = Some(buffer);
    }

    fn set_layout(&self) -> pso::DescriptorSetLayoutBinding {
        pso::DescriptorSetLayoutBinding {
            binding: self.binding,
            ty: pso::DescriptorType::UniformBuffer,
            count: 1,
            stage_flags: self.stage,
            immutable_samplers: false,
        }
    }

    fn set_write<'a>(
        &'a self,
        set: &'a B::DescriptorSet,
    ) -> pso::DescriptorSetWrite<'a, B, Option<pso::Descriptor<'a, B>>> {
        let buffer = self.buffer.as_ref().expect("Uniform: Missing Buffer!");

        pso::DescriptorSetWrite {
            set,
            binding: self.binding,
            array_offset: 0,
            descriptors: Some(pso::Descriptor::Buffer(buffer.raw(), None..None)),
        }
    }
}
