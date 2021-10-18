use crate::gfx::uniform::{Uniform, UniformArray};
use crate::gfx::wgpu::TextureView;

use std::num::NonZeroU32;
use std::ops::RangeBounds;

pub struct BindingType(wgpu::BindingType);

impl From<BindingType> for wgpu::BindingType {
    fn from(val: BindingType) -> Self {
        val.0
    }
}

impl From<wgpu::BindingType> for BindingType {
    fn from(val: wgpu::BindingType) -> Self {
        BindingType(val)
    }
}

impl<T: Copy> From<&Uniform<T>> for BindingType {
    fn from(val: &Uniform<T>) -> Self {
        BindingType(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(std::num::NonZeroU64::new(val.size()).unwrap()),
        })
    }
}

impl From<&TextureView> for BindingType {
    fn from(val: &TextureView) -> Self {
        BindingType(wgpu::BindingType::Texture {
            view_dimension: val.dimension,
            sample_type: val.format.describe().sample_type,
            multisampled: val.sample_count > 1,
        })
    }
}

/// A type aimed at simplifying the creation of a bind group layout.
#[derive(Debug)]
pub struct LayoutBuilder<'l> {
    label: &'l str,
    bindings: Vec<(wgpu::ShaderStages, wgpu::BindingType, Option<NonZeroU32>)>,
}

impl<'l> LayoutBuilder<'l> {
    /// Begin building the bind group layout.
    pub fn new(label: &'l str) -> Self {
        Self {
            label,
            bindings: Vec::new(),
        }
    }

    /// Specify a new binding.
    ///
    /// The `binding` position of each binding will be inferred as the index within the order that
    /// they are added to this builder type. If you require manually specifying the binding
    /// location, you may be better off not using the `BindGroupLayoutBuilder` and instead
    /// constructing the `BindGroupLayout` and `BindGroup` manually.
    pub fn binding(
        mut self,
        visibility: wgpu::ShaderStages,
        ty: BindingType,
        count: Option<u32>,
    ) -> Self {
        self.bindings.push((
            visibility,
            ty.into(),
            count.map(|n| NonZeroU32::new(n).unwrap()),
        ));
        self
    }

    pub fn uniform(self, visibility: wgpu::ShaderStages) -> Self {
        self.binding(visibility, wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            min_binding_size: None,
            has_dynamic_offset: false,
        }.into(), None)
    }

    pub fn uniform_array(self, visibility: wgpu::ShaderStages) -> Self {
        self.uniform(visibility)
    }

    /// Add a sampler binding to the layout.
    pub fn sampler(self, visibility: wgpu::ShaderStages) -> Self {
        self.binding(
            visibility,
            wgpu::BindingType::Sampler { comparison: false, filtering: true }.into(),
            None,
        )
    }

    /// Add a sampler binding to the layout.
    pub fn comparison_sampler(self, visibility: wgpu::ShaderStages) -> Self {
        self.binding(visibility, wgpu::BindingType::Sampler { comparison: true, filtering: true }.into(), None)
    }

    // TODO: fix this and add texture_from or just From<thing> for parameter?
    pub fn array_tex(self, visibility: wgpu::ShaderStages) -> Self {
        self.binding(visibility, BindingType(wgpu::BindingType::Texture {
            view_dimension: wgpu::TextureViewDimension::D2Array,
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            multisampled: false,
        }), None)
    }
    pub fn tex(self, visibility: wgpu::ShaderStages) -> Self {
        self.binding(visibility, BindingType(wgpu::BindingType::Texture {
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            multisampled: false,
        }), None)
    }

    pub fn texture(self, visibility: wgpu::ShaderStages, view: &TextureView) -> Self {
        self.binding(visibility, view.into(), None)
    }
    pub fn textures(self, visibility: wgpu::ShaderStages, n: usize) -> Self {
        self.binding(visibility, BindingType(wgpu::BindingType::Texture {
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            multisampled: false,
        }), Some(n as u32))
    }

    /// Add a storage buffer binding to the layout.
    // pub fn storage_buffer(
    //     self,
    //     visibility: wgpu::ShaderStage,
    //     dynamic: bool,
    //     readonly: bool,
    // ) -> Self {
    //     let ty = wgpu::BindingType::StorageBuffer { dynamic, readonly };
    //     self.binding(visibility, ty)
    // }

    /// Add a storage texture binding to the layout.
    // pub fn storage_texture(
    //     self,
    //     visibility: wgpu::ShaderStage,
    //     format: wgpu::TextureFormat,
    //     dimension: wgpu::TextureViewDimension,
    //     component_type: wgpu::TextureComponentType,
    //     readonly: bool,
    // ) -> Self {
    //     let ty = wgpu::BindingType::StorageTexture {
    //         dimension,
    //         component_type,
    //         format,
    //         readonly,
    //     };
    //     self.binding(visibility, ty)
    // }

    /// Short-hand for adding a storage texture binding for a full view of the given texture to the
    /// layout.
    ///
    /// The `format`, `dimension` and `component_type` are inferred from the given `texture`.
    // pub fn storage_texture_from(
    //     self,
    //     visibility: wgpu::ShaderStage,
    //     texture: &wgpu::Texture,
    //     readonly: bool,
    // ) -> Self {
    //     self.storage_texture(
    //         visibility,
    //         texture.format(),
    //         texture.view_dimension(),
    //         texture.component_type(),
    //         readonly,
    //     )
    // }

    /// Build the bind group layout from the specified parameters.
    pub fn build(self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let bindings = self
            .bindings
            .into_iter()
            .enumerate()
            .map(|(i, (visibility, ty, count))| wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility,
                ty,
                count,
            })
            .collect::<Vec<_>>();

        let label = &format!("{}_layout", self.label);

        let descriptor = wgpu::BindGroupLayoutDescriptor {
            label: Some(label),
            entries: &bindings,
        };

        device.create_bind_group_layout(&descriptor)
    }
}

/// Simplified creation of a bind group.
#[derive(Debug)]
pub struct Builder<'l, 'a> {
    label: &'l str,
    resources: Vec<wgpu::BindingResource<'a>>,
}

impl<'l, 'a> Builder<'l, 'a> {
    /// Begin building the bind group.
    pub fn new(label: &'l str) -> Self {
        Self {
            label,
            resources: Vec::new(),
        }
    }

    /// Specify a new binding.
    ///
    /// The `binding` position of each binding will be inferred as the index within the order that
    /// they are added to this builder type. If you require manually specifying the binding
    /// location, you may be better off not using the `BindGroupBuilder` and instead constructing
    /// the `BindGroupLayout` and `BindGroup` manually.
    pub fn binding(mut self, resource: wgpu::BindingResource<'a>) -> Self {
        self.resources.push(resource);
        self
    }

    /// Specify a slice of a buffer to be bound.
    ///
    /// The given `range` represents the start and end point of the buffer to be bound in bytes.
    pub fn buffer<T, S>(self, buffer: &'a wgpu::Buffer, bounds: S) -> Self
    where
        T: Copy,
        S: RangeBounds<wgpu::BufferAddress>,
    {
        use std::ops::Bound;
        let offset = match bounds.start_bound() {
            Bound::Included(&bound) => bound,
            Bound::Excluded(&bound) => bound + 1,
            Bound::Unbounded => 0,
        };
        let size = match bounds.end_bound() {
            Bound::Included(&bound) => Some(bound + 1 - offset),
            Bound::Excluded(&bound) => Some(bound - offset),
            Bound::Unbounded => None,
        }
        .map(|size| wgpu::BufferSize::new(size).expect("Buffer slices can not be empty"));

        self.binding(wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer,
            offset,
            size
        }))
    }

    /// Specify a slice of a buffer of elements of type `T` to be bound.
    ///
    /// This method is similar to `buffer_bytes`, but expects a range of **elements** rather than a
    /// range of **bytes**.
    ///
    /// Type `T` *must* be either `#[repr(C)]` or `#[repr(transparent)]`.
    // pub fn buffer<T, S>(self, buffer: &'a wgpu::Buffer, range: S) -> Self
    // where
    //     T: Copy,
    //     S: RangeBounds<wgpu::BufferAddress>,
    // {
    //     let size_bytes = std::mem::size_of::<T>() as wgpu::BufferAddress;
    //     let start = range.start as wgpu::BufferAddress * size_bytes;
    //     let end = range.end as wgpu::BufferAddress * size_bytes;
    //     let byte_range = start..end;
    //     self.buffer_bytes(buffer, byte_range)
    // }

    pub fn uniform<T: Copy>(self, uniform: &'a Uniform<T>) -> Self {
        self.buffer::<T, _>(&uniform.buffer, ..)
    }

    pub fn uniform_array<T: Copy>(self, uniform: &'a UniformArray<T>) -> Self {
        self.buffer::<T, _>(&uniform.buffer, ..)
    }

    /// Specify a sampler to be bound.
    pub fn sampler(self, sampler: &'a wgpu::Sampler) -> Self {
        let resource = wgpu::BindingResource::Sampler(sampler);
        self.binding(resource)
    }

    /// Specify a texture view to be bound.
    pub fn texture(self, view: &'a TextureView) -> Self {
        let resource = wgpu::BindingResource::TextureView(view);
        self.binding(resource)
    }
    // FIXME: Automatically ref this stuff to make calling easier
    /// Specify a texture view to be bound.
    pub fn textures(self, views: &'a [&'a wgpu::TextureView]) -> Self {
        let resource = wgpu::BindingResource::TextureViewArray(views);
        self.binding(resource)
    }

    /// Build the bind group with the specified resources.
    pub fn build(self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        let bindings = self
            .resources
            .into_iter()
            .enumerate()
            .map(|(i, resource)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource,
            })
            .collect::<Vec<_>>();

        let label = &format!("{}_group", self.label);

        let descriptor = wgpu::BindGroupDescriptor {
            label: Some(label),
            layout,
            entries: &bindings,
        };
        device.create_bind_group(&descriptor)
    }
}
