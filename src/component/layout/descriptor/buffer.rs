/*
use super::{DescriptorBinding, ShaderStageFlags};
use glsl_layout::AsStd140;
use rendy::{
    factory::Factory,
    hal::{self, pso, Backend},
    resource::{Buffer, BufferInfo, Escape},
};
use std::marker::PhantomData;

pub trait UniformInfo {
    fn binding(&self) -> DescriptorBinding;
    fn stage(&self) -> pso::ShaderStageFlags;
    fn buffer<B: Backend>(&self) -> Box<dyn Uniform<B>>;
}

pub struct UniformBufferInfo<T> {
    binding: DescriptorBinding,
    stage: pso::ShaderStageFlags,
    marker: PhantomData<T>,
}

impl<T> UniformInfo for UniformBufferInfo<T> {
    fn binding(&self) -> DescriptorBinding {
        self.binding
    }

    fn stage(&self) -> ShaderStageFlags {
        self.stage
    }

    fn buffer<B: Backend>(&self) -> Box<dyn Uniform<B>> {
        Box::new(UniformBuffer::<B, T>::new(self))
    }
}

pub trait Uniform<B: Backend> {
    fn binding(&self) -> u32;
    fn alloc(&mut self, factory: &Factory<B>);
    fn set_layout(&self) -> pso::DescriptorSetLayoutBinding;
    fn set_write<'a>(
        &'a self,
        set: &'a <B as Backend>::DescriptorSet,
    ) -> pso::DescriptorSetWrite<'a, B, Option<pso::Descriptor<'a, B>>>;
}

pub struct UniformBuffer<B: Backend, T: AsStd140>
where
    T::Std140: Sized,
{
    pub info: Box<dyn UniformInfo>,
    marker: PhantomData<T>,

    buffer: Option<Escape<Buffer<B>>>,
}

impl<B: Backend, T: AsStd140> UniformBuffer<B, T>
where
    T::Std140: Sized,
{
    pub fn new(info: Box<dyn UniformInfo>) -> Self {
        Self {
            info,
            marker: PhantomData,
            buffer: None,
        }
    }

    fn size() -> usize {
        std::mem::size_of::<<T as AsStd140>::Std140>()
    }
}

impl<B: Backend, T: AsStd140> Uniform<B> for UniformBuffer<B, T>
where
    T::Std140: Sized,
{
    fn binding(&self) -> DescriptorBinding {
        self.info.binding
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
            binding: self.info.binding,
            ty: pso::DescriptorType::UniformBuffer,
            count: 1,
            stage_flags: self.info.stage,
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
            binding: self.info.binding,
            array_offset: 0,
            descriptors: Some(pso::Descriptor::Buffer(buffer.raw(), None..None)),
        }
    }
}
*/
