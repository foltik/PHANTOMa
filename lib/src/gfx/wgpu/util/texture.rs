use crate::gfx::wgpu;

#[derive(Debug)]
pub struct TextureBuilder<'l> {
    label: &'l str,
    descriptor: wgpu::TextureDescriptor<'static>,
}

impl<'l> TextureBuilder<'l> {
    pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub const DEFAULT_DESCRIPTOR: wgpu::TextureDescriptor<'static> = wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: 1920, height: 1080, depth: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: Self::COLOR_FORMAT,
        usage: wgpu::TextureUsage::empty(),
    };

    pub fn new(label: &'l str) -> Self {
        Self { label, descriptor: Self::DEFAULT_DESCRIPTOR }
    }

    /// Creates a new `Default` builder
    pub fn new_color(label: &'l str) -> Self {
        Self::new(label)
    }

    pub fn new_depth(label: &'l str) -> Self {
        Self::new(label).format(Self::DEPTH_FORMAT)
    }

    /// Specify the width and height of the texture.
    ///
    /// Note: On calls to `size`, `depth` and `extent` the `Builder` will attempt to infer the
    /// `wgpu::TextureDimension` of its inner `wgpu::TextureDescriptor` by examining its `size`
    /// field.
    pub fn size(mut self, [width, height, depth]: [u32; 3]) -> Self {
        self.descriptor.size.width = width;
        self.descriptor.size.height = height;
        self.descriptor.size.depth = depth;
        self.infer_dimension_from_size();
        self
    }

    pub fn mip_level_count(mut self, count: u32) -> Self {
        self.descriptor.mip_level_count = count;
        self
    }

    /// Specify the number of samples per pixel in the case that the texture is multisampled.
    pub fn sample_count(mut self, count: u32) -> Self {
        self.descriptor.sample_count = count;
        self
    }

    /// Specify the texture format.
    pub fn format(mut self, format: wgpu::TextureFormat) -> Self {
        self.descriptor.format = format;
        self
    }

    /// Describes to the implementation how the texture is to be used.
    ///
    /// It is important that the set of usage bits reflects the
    pub fn usage(mut self, usage: wgpu::TextureUsage) -> Self {
        self.descriptor.usage = usage;
        self
    }

    // If `depth` is greater than `1` then `D3` is assumed, otherwise if `height` is greater than
    // `1` then `D2` is assumed, otherwise `D1` is assumed.
    fn infer_dimension_from_size(&mut self) {
        if self.descriptor.size.depth > 1 {
            self.descriptor.dimension = wgpu::TextureDimension::D3;
        } else if self.descriptor.size.height > 1 {
            self.descriptor.dimension = wgpu::TextureDimension::D2;
        } else {
            self.descriptor.dimension = wgpu::TextureDimension::D1;
        }
    }

    /// Build the texture resulting from the specified parameters with the given device.
    pub fn build(self, device: &wgpu::Device) -> wgpu::Texture {
        let mut descriptor = self.descriptor;

        let label = &format!("{}_texture", self.label);
        descriptor.label = Some(&label);

        wgpu::Texture {
            texture: device.create_texture(&descriptor),
            label: self.label.to_owned(),
            size: descriptor.size,
            dimension: descriptor.dimension,
            format: descriptor.format,
            mip_count: descriptor.mip_level_count,
            sample_count: descriptor.sample_count,
        }
    }
}

pub struct TextureViewBuilder<'a> {
    texture: &'a wgpu::Texture,
    descriptor: wgpu::TextureViewDescriptor<'static>,
}

impl<'a> TextureViewBuilder<'a> {
    pub const DEFAULT_DESCRIPTOR: wgpu::TextureViewDescriptor<'static> = wgpu::TextureViewDescriptor {
        label: None,
        format: None,
        dimension: None,
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    };

    pub fn new(texture: &'a wgpu::Texture) -> Self {
        let mut descriptor = Self::DEFAULT_DESCRIPTOR;
        descriptor.format = Some(texture.format);
        descriptor.dimension = Some(match texture.dimension {
            wgpu::TextureDimension::D1 => wgpu::TextureViewDimension::D1,
            wgpu::TextureDimension::D2 => wgpu::TextureViewDimension::D2,
            wgpu::TextureDimension::D3 => wgpu::TextureViewDimension::D3,
        });
        Self { texture, descriptor }
    }

    pub fn format(mut self, format: wgpu::TextureFormat) -> Self {
        self.descriptor.format = Some(format);
        self
    }

    pub fn dimension(mut self, dimension: wgpu::TextureViewDimension) -> Self {
        self.descriptor.dimension = Some(dimension);
        self
    }

    pub fn aspect(mut self, aspect: wgpu::TextureAspect) -> Self {
        self.descriptor.aspect = aspect;
        self
    }

    pub fn base_mip_level(mut self, base_mip_level: u32) -> Self {
        self.descriptor.base_mip_level = base_mip_level;
        self
    }

    pub fn level_count(mut self, level_count: u32) -> Self {
        self.descriptor.level_count = Some(std::num::NonZeroU32::new(level_count).unwrap());
        self
    }

    /// Short-hand for specifying a **TextureView** for a single given mip level.
    ///
    /// In other words, this is short-hand for the following:
    ///
    /// ```ignore
    /// builder
    ///     .base_mip_level(level)
    ///     .level_count(1)
    /// ```
    pub fn level(self, level: u32) -> Self {
        self.base_mip_level(level).level_count(1)
    }

    pub fn base_array_layer(mut self, base_array_layer: u32) -> Self {
        self.descriptor.base_array_layer = base_array_layer;
        self
    }

    pub fn array_layer_count(mut self, array_layer_count: u32) -> Self {
        self.descriptor.array_layer_count = Some(std::num::NonZeroU32::new(array_layer_count).unwrap());
        self
    }

    /// Short-hand for specifying a **TextureView** for a single given base array layer.
    ///
    /// In other words, this is short-hand for the following:
    ///
    /// ```ignore
    /// builder
    ///     .base_array_layer(layer)
    ///     .array_layer_count(1)
    /// ```
    pub fn layer(self, layer: u32) -> Self {
        self.base_array_layer(layer).array_layer_count(1)
    }

    pub fn build(self) -> wgpu::TextureView {
        let mut descriptor = self.descriptor;

        let label = &format!("{}_view", &self.texture.label);
        descriptor.label = Some(label);

        wgpu::TextureView {
            view: self.texture.create_view(&descriptor),
            size: self.texture.size,
            dimension: descriptor.dimension.unwrap(),
            format: self.texture.format,
            mip_count: self.texture.mip_count,
            sample_count: self.texture.sample_count,
        }
    }
}

