use rendy::{
    factory::Factory,
    graph::{GraphContext, NodeImage},
    hal::{self, pso, Backend},
    resource::{Escape, Handle, Image, ImageView, ImageViewInfo, Sampler as RSampler, ViewKind},
};
use super::{Descriptor, DescriptorBinding, ShaderStageFlags};

pub use rendy::resource::SamplerDesc;

pub struct Sampler<B: Backend> {
    pub binding: DescriptorBinding,
    pub stage: pso::ShaderStageFlags,
    desc: SamplerDesc,

    image_info: Option<ImageViewInfo>,
    image: Option<Handle<Image<B>>>,

    view: Option<Escape<ImageView<B>>>,
    sampler: Option<Escape<RSampler<B>>>,
}

impl<B: Backend> Sampler<B> {
    pub fn new(binding: DescriptorBinding, stage: ShaderStageFlags, desc: SamplerDesc) -> Self {
        Self {
            binding,
            desc,
            stage,
            image_info: None,
            image: None,
            sampler: None,
            view: None,
        }
    }

    pub fn with_node_image(mut self, ctx: &GraphContext<B>, image: &NodeImage) -> Self {
        let handle = ctx.get_image(image.id).unwrap();

        let info = ImageViewInfo {
            view_kind: ViewKind::D2,
            format: handle.info().format,
            swizzle: hal::format::Swizzle::NO,
            range: image.range.clone(),
        };

        self.image_info = Some(info);
        self.image = Some(Handle::clone(handle));
        self
    }
}

impl<B: Backend> Descriptor<B> for Sampler<B> {
    fn binding(&self) -> DescriptorBinding {
        self.binding
    }

    fn alloc(&mut self, factory: &Factory<B>) {
        let image_info = self.image_info.as_ref().expect("Sampler: Missing image!");
        let image = self.image.as_ref().expect("Sampler: Missing image!");

        let view = factory
            .create_image_view(Handle::clone(image), image_info.clone())
            .unwrap();

        let sampler = factory.create_sampler(self.desc.clone()).unwrap();

        self.view = Some(view);
        self.sampler = Some(sampler);
    }

    fn set_layout(&self) -> pso::DescriptorSetLayoutBinding {
        pso::DescriptorSetLayoutBinding {
            binding: self.binding,
            ty: pso::DescriptorType::CombinedImageSampler,
            count: 1,
            stage_flags: self.stage,
            immutable_samplers: false,
        }
    }

    fn set_write<'a>(
        &'a self,
        set: &'a B::DescriptorSet,
    ) -> pso::DescriptorSetWrite<'a, B, Option<pso::Descriptor<'a, B>>> {
        let view = self.view.as_ref().expect("Sampler: Missing ImageView!");
        let sampler = self.sampler.as_ref().expect("Sampler: Missing Sampler!");

        pso::DescriptorSetWrite {
            set,
            binding: self.binding,
            array_offset: 0,
            descriptors: Some(pso::Descriptor::CombinedImageSampler(
                view.raw(),
                hal::image::Layout::ShaderReadOnlyOptimal,
                sampler.raw(),
            )),
        }
    }
}
