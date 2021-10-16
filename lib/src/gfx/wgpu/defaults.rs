use std::default::Default;
use super::*;

// Instance defaults

pub fn power_preference() -> PowerPreference {
    PowerPreference::HighPerformance
}

/// Nannou's default WGPU backend preferences.
pub const fn backends() -> Backends {
    Backends::PRIMARY
}

/// The default set of `Features` used within the `default_device_descriptor()` function.
pub fn features() -> Features {
    // allow uniform texture2D textures[]
    Features::TEXTURE_BINDING_ARRAY |
    Features::UNSIZED_BINDING_ARRAY |

    Features::SPIRV_SHADER_PASSTHROUGH
}

pub fn limits() -> Limits {
    Limits {
        max_sampled_textures_per_shader_stage: 32,
        ..Default::default()
    }
}


// Device defaults

pub fn device_descriptor() -> DeviceDescriptor<'static> {
    DeviceDescriptor {
        label: None,
        features: features(),
        limits: limits(),
    }
}


// Window defaults

pub const fn texture_format() -> TextureFormat {
    TextureFormat::Bgra8UnormSrgb
}
pub const fn depth_format() -> TextureFormat {
    TextureFormat::Depth32Float
}
pub const fn index_format() -> IndexFormat {
    IndexFormat::Uint32
}