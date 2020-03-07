use core::marker::PhantomData;
use glsl_layout::AsStd140;
use rendy::{
    command::RenderPassEncoder,
    factory::Factory,
    graph::{GraphContext, NodeImage},
    hal::{self, device::Device, pso, Backend},
    memory::{MappedRange, Write},
    resource::{
        Buffer, BufferInfo, DescriptorSet, DescriptorSetLayout, Escape, Handle, ImageViewInfo,
        SamplerDesc, ViewKind,
    },
};

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

    #[allow(dead_code)]
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
pub struct DynamicUniform<B: Backend, T: Sized> {
    layout: Handle<DescriptorSetLayout<B>>,
    per_image: Vec<PerImageDynamicUniform<B, T>>,
}

#[derive(Debug)]
struct PerImageDynamicUniform<B: Backend, T: Sized> {
    buffer: Escape<Buffer<B>>,
    set: Escape<DescriptorSet<B>>,
    marker: PhantomData<T>,
}

impl<B: Backend, T: Sized> DynamicUniform<B, T> {
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

    pub fn write_raw(&mut self, factory: &Factory<B>, index: usize, bytes: &[u8]) -> bool {
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
                .write::<u8>(factory.device(), 0..std::mem::size_of::<T>() as u64)
                .unwrap()
        };

        let slice = unsafe { writer.slice() };
        slice.copy_from_slice(bytes);

        changed
    }

    /// Write `T` to this descriptor set memory
    pub fn write(&mut self, factory: &Factory<B>, index: usize, item: &T) -> bool {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                item as *const T as *const u8,
                std::mem::size_of::<T>(),
            )
        };

        self.write_raw(factory, index, bytes)
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

impl<B: Backend, T: Sized> PerImageDynamicUniform<B, T> {
    fn new(factory: &Factory<B>, layout: &Handle<DescriptorSetLayout<B>>) -> Self {
        let buffer = factory
            .create_buffer(
                BufferInfo {
                    size: std::mem::size_of::<T>() as u64,
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

#[derive(Debug)]
pub struct Sampler<B: Backend> {
    set: Escape<DescriptorSet<B>>,
}

impl<B: Backend> Sampler<B> {
    pub fn new(
        ctx: &GraphContext<B>,
        factory: &Factory<B>,
        image: &NodeImage,
        info: SamplerDesc,
        flags: pso::ShaderStageFlags,
    ) -> Self {
        let layout_binding: Handle<_> = factory
            .create_descriptor_set_layout(vec![pso::DescriptorSetLayoutBinding {
                binding: 0,
                ty: pso::DescriptorType::CombinedImageSampler,
                count: 1,
                stage_flags: flags,
                immutable_samplers: false,
            }])
            .unwrap()
            .into();

        let sampler = factory.create_sampler(info).unwrap();

        let handle = ctx.get_image(image.id).unwrap();

        let view = factory
            .create_image_view(
                handle.clone(),
                ImageViewInfo {
                    view_kind: ViewKind::D2,
                    format: handle.info().format,
                    swizzle: hal::format::Swizzle::NO,
                    range: image.range.clone(),
                },
            )
            .unwrap();

        let set = factory
            .create_descriptor_set(layout_binding.clone())
            .unwrap();

        let desc = pso::Descriptor::CombinedImageSampler(view.raw(), image.layout, sampler.raw());

        unsafe {
            factory.write_descriptor_sets(Some(pso::DescriptorSetWrite {
                set: set.raw(),
                binding: 0,
                array_offset: 0,
                descriptors: Some(desc),
            }));
        }

        Self { set }
    }

    pub fn bind(
        &self,
        pipeline_layout: &B::PipelineLayout,
        binding: u32,
        encoder: &mut RenderPassEncoder<B>,
    ) {
        unsafe {
            encoder.bind_graphics_descriptor_sets(
                pipeline_layout,
                binding,
                Some(self.set.raw()),
                std::iter::empty(),
            );
        }
    }
}
