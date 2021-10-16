// FIXME: Add border color builder param

/// Simplifies the construction of a `Sampler` with a set of reasonable defaults.
#[derive(Debug)]
pub struct SamplerBuilder<'l> {
    label: &'l str,
    pub descriptor: wgpu::SamplerDescriptor<'static>,
}

impl<'l> SamplerBuilder<'l> {
    pub const DEFAULT_DESCRIPTOR: wgpu::SamplerDescriptor<'static> = wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        border_color: None,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        lod_min_clamp: -100.0,
        lod_max_clamp: 100.0,
        anisotropy_clamp: None,
        compare: None,
    };

    /// Begin building a `Sampler`, starting with the `Default` parameters.
    pub fn new(label: &'l str) -> Self {
        Self { label, descriptor: Self::DEFAULT_DESCRIPTOR }
    }

    /// How the implementation should behave when sampling outside of the texture coordinates range
    /// [0.0, 1.0].
    pub fn address_mode_u(mut self, mode: wgpu::AddressMode) -> Self {
        self.descriptor.address_mode_u = mode;
        self
    }

    /// How the implementation should behave when sampling outside of the texture coordinates range
    /// [0.0, 1.0].
    pub fn address_mode_v(mut self, mode: wgpu::AddressMode) -> Self {
        self.descriptor.address_mode_v = mode;
        self
    }

    /// How the implementation should behave when sampling outside of the texture coordinates range
    /// [0.0, 1.0].
    pub fn address_mode_w(mut self, mode: wgpu::AddressMode) -> Self {
        self.descriptor.address_mode_w = mode;
        self
    }

    /// How the implementation should behave when sampling outside of the texture coordinates range
    /// [0.0, 1.0].
    ///
    /// Applies the same address mode to all axes.
    pub fn address_mode(self, mode: wgpu::AddressMode) -> Self {
        self.address_mode_u(mode)
            .address_mode_v(mode)
            .address_mode_w(mode)
    }

    /// How the implementation should sample from the image when it is respectively larger than the
    /// original.
    pub fn mag_filter(mut self, filter: wgpu::FilterMode) -> Self {
        self.descriptor.mag_filter = filter;
        self
    }

    /// How the implementation should sample from the image when it is respectively smaller than
    /// the original.
    pub fn min_filter(mut self, filter: wgpu::FilterMode) -> Self {
        self.descriptor.min_filter = filter;
        self
    }

    /// How the implementation should choose which mipmap to use.
    pub fn mipmap_filter(mut self, filter: wgpu::FilterMode) -> Self {
        self.descriptor.mipmap_filter = filter;
        self
    }

    /// The minimum mipmap level to use.
    pub fn lod_min_clamp(mut self, min: f32) -> Self {
        self.descriptor.lod_min_clamp = min;
        self
    }

    /// The maximum mipmap level to use.
    pub fn lod_max_clamp(mut self, max: f32) -> Self {
        self.descriptor.lod_max_clamp = max;
        self
    }

    pub fn compare(mut self, f: wgpu::CompareFunction) -> Self {
        self.descriptor.compare = Some(f);
        self
    }

    pub fn build(self, device: &wgpu::Device) -> wgpu::Sampler {
        let mut descriptor = self.descriptor;

        let label = format!("{}_sampler", self.label);
        descriptor.label = Some(&label);

        device.create_sampler(&descriptor)
    }
}