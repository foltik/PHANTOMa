use rendy::{
    command::RenderPassEncoder,
    factory::Factory,
    hal::{self, device::Device, pso, Backend},
    memory::{MappedRange, Write},
    resource::{
        Buffer, BufferInfo, DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle,
    },
};

use glsl_layout::AsStd140;

use core::marker::PhantomData;

#[derive(Debug)]
pub struct PushConstant<T: AsStd140>
where
    T::Std140: Sized,
{
    item: T,
    offset: u32,
    stages: pso::ShaderStageFlags,
}

impl<T: AsStd140> std::ops::Deref for PushConstant<T>
where
    T::Std140: Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T: AsStd140> std::ops::DerefMut for PushConstant<T>
where
    T::Std140: Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<T: AsStd140> PushConstant<T>
where
    T::Std140: Sized,
{
    pub fn new(item: T, offset: u32, stages: pso::ShaderStageFlags) -> Self {
        Self {
            item,
            offset,
            stages,
        }
    }

    pub fn bind<B: Backend>(&self, layout: &B::PipelineLayout, encoder: &mut RenderPassEncoder<B>) {
        let std = &self.item.std140();

        // TODO: Assert the layout size is the same as our Std140?

        let raw = unsafe {
            std::slice::from_raw_parts(
                std as *const <T as AsStd140>::Std140 as *const u32,
                std::mem::size_of::<<T as AsStd140>::Std140>() / std::mem::size_of::<u32>(),
            )
        };

        unsafe {
            encoder.push_constants(layout, self.stages, self.offset, raw);
        }
    }
}

/// Provides per-image abstraction for an arbitrary `DescriptorSet`.
#[derive(Debug)]
pub struct DynamicUniform<B: Backend, T: AsStd140>
where
    T::Std140: Sized,
{
    layout: RendyHandle<DescriptorSetLayout<B>>,
    per_image: Vec<PerImageDynamicUniform<B, T>>,
}

#[derive(Debug)]
struct PerImageDynamicUniform<B: Backend, T: AsStd140>
where
    T::Std140: Sized,
{
    buffer: Escape<Buffer<B>>,
    set: Escape<DescriptorSet<B>>,
    marker: PhantomData<T>,
}

impl<B: Backend, T: AsStd140> DynamicUniform<B, T>
where
    T::Std140: Sized,
{
    /// Create a new `DynamicUniform`, allocating descriptor set memory using the provided `Factory`
    pub fn new(factory: &Factory<B>, flags: pso::ShaderStageFlags) -> Self {
        let layout_binding = factory
            .create_descriptor_set_layout(vec![pso::DescriptorSetLayoutBinding {
                binding: 0,
                ty: pso::DescriptorType::UniformBuffer,
                count: 1,
                stage_flags: flags,
                immutable_samplers: false,
            }])
            .unwrap();

        Self {
            layout: layout_binding.into(),
            per_image: Vec::new(),
        }
    }

    /// Returns the `DescriptSetLayout` for this set.
    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.layout.raw()
    }

    /// Write `T` to this descriptor set memory
    pub fn write(&mut self, factory: &Factory<B>, index: usize, item: &T::Std140) -> bool {
        let mut changed = false;
        let this_image = {
            while self.per_image.len() <= index {
                self.per_image
                    .push(PerImageDynamicUniform::new(factory, &self.layout));
                changed = true;
            }
            &mut self.per_image[index]
        };

        let mut mapped = this_image.map(factory);
        let mut writer = unsafe {
            mapped
                .write::<u8>(factory.device(), 0..std::mem::size_of::<T::Std140>() as u64)
                .unwrap()
        };
        let slice = unsafe { writer.slice() };

        let bytes = unsafe {
            std::slice::from_raw_parts(
                item as *const T::Std140 as *const u8,
                std::mem::size_of::<T::Std140>(),
            )
        };

        slice.copy_from_slice(bytes);
        changed
    }

    #[inline]
    pub fn bind(
        &self,
        index: usize,
        pipeline_layout: &B::PipelineLayout,
        binding_id: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) {
        self.per_image[index].bind(pipeline_layout, binding_id, encoder);
    }
}

impl<B: Backend, T: AsStd140> PerImageDynamicUniform<B, T>
where
    T::Std140: Sized,
{
    fn new(factory: &Factory<B>, layout: &RendyHandle<DescriptorSetLayout<B>>) -> Self {
        let buffer = factory
            .create_buffer(
                BufferInfo {
                    size: std::mem::size_of::<T::Std140>() as u64,
                    usage: hal::buffer::Usage::UNIFORM,
                },
                rendy::memory::Dynamic,
            )
            .unwrap();

        let set = factory.create_descriptor_set(layout.clone()).unwrap();
        let desc = pso::Descriptor::Buffer(buffer.raw(), None..None);
        unsafe {
            let set = set.raw();
            factory.write_descriptor_sets(Some(pso::DescriptorSetWrite {
                set,
                binding: 0,
                array_offset: 0,
                descriptors: Some(desc),
            }));
        }
        Self {
            buffer,
            set,
            marker: PhantomData,
        }
    }

    fn map(&mut self, factory: &Factory<B>) -> MappedRange<B> {
        let range = 0..self.buffer.size();
        self.buffer.map(factory.device(), range).unwrap()
    }

    #[inline]
    fn bind(
        &self,
        pipeline_layout: &B::PipelineLayout,
        set_id: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) {
        unsafe {
            encoder.bind_graphics_descriptor_sets(
                pipeline_layout,
                set_id,
                Some(self.set.raw()),
                std::iter::empty(),
            );
        }
    }
}
