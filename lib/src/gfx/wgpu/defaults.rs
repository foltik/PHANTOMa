use std::default::Default;
use super::*;

// Instance defaults

pub fn power_preference() -> PowerPreference {
    PowerPreference::HighPerformance
}

/// Nannou's default WGPU backend preferences.
pub fn backends() -> BackendBit {
    BackendBit::PRIMARY
}

/// The default set of `Features` used within the `default_device_descriptor()` function.
pub fn features() -> Features {
    Features::SAMPLED_TEXTURE_BINDING_ARRAY
}



// Device defaults

pub fn device_descriptor() -> DeviceDescriptor {
    DeviceDescriptor {
        features: features(),
        limits: wgpu::Limits {
            max_sampled_textures_per_shader_stage: 32,
            ..Default::default()
        },
        shader_validation: true,
    }
}


// Window defaults

pub fn texture_format() -> TextureFormat {
    TextureFormat::Bgra8UnormSrgb
}

pub fn depth_format() -> TextureFormat {
    TextureFormat::Depth32Float
}