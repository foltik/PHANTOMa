/*
use rendy::{
    factory::Factory,
    graph::{GraphContext, NodeImage},
    hal::{self, pso, Backend},
    resource::{Escape, Handle, Image, ImageView, ImageViewInfo, Sampler as RSampler, ViewKind},
};
use super::{DescriptorBinding, ShaderStageFlags};

pub use rendy::resource::SamplerDesc;

pub struct SamplerInfo {
    binding: DescriptorBinding,
    stage: pso::ShaderStageFlags,
    desc: SamplerDesc,
}

pub struct Sampler<B: Backend> {
    info: SamplerInfo,

    image_info: Option<ImageViewInfo>,
    image: Option<Handle<Image<B>>>,

    view: Option<Escape<ImageView<B>>>,
    sampler: Option<Escape<RSampler<B>>>,
}

impl<B: Backend> Sampler<B> {
    pub fn new(info: SamplerInfo) -> Self {
        Self {
            info,
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

    fn binding(&self) -> DescriptorBinding {
        self.binding
    }

    fn alloc(&mut self, factory: &Factory<B>) {
        let image_info = self.image_info.as_ref().expect("Sampler: Missing image!");
        let image = self.image.as_ref().expect("Sampler: Missing image!");

        let view = factory
            .create_image_view(Handle::clone(image), image_info.clone())
            .unwrap();

        let sampler = factory.create_sampler(self.info.desc.clone()).unwrap();

        self.view = Some(view);
        self.sampler = Some(sampler);
    }

    fn set_layout(&self) -> pso::DescriptorSetLayoutBinding {
        pso::DescriptorSetLayoutBinding {
            binding: self.info.binding,
            ty: pso::DescriptorType::CombinedImageSampler,
            count: 1,
            stage_flags: self.info.stage,
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
            binding: self.info.binding,
            array_offset: 0,
            descriptors: Some(pso::Descriptor::CombinedImageSampler(
                view.raw(),
                hal::image::Layout::ShaderReadOnlyOptimal,
                sampler.raw(),
            )),
        }
    }
}
*/
