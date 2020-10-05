use std::default::Default;

use super::*;

/// The default power preference used for requesting the WGPU adapter.
pub fn power_preference() -> PowerPreference {
    PowerPreference::HighPerformance
}

/// Nannou's default WGPU backend preferences.
pub fn backends() -> BackendBit {
    BackendBit::PRIMARY
}

/// The default set of `Features` used within the `default_device_descriptor()` function.
pub fn features() -> Features {
    Default::default()
}

pub fn device_descriptor() -> DeviceDescriptor {
    DeviceDescriptor {
        features: features(),
        limits: Default::default(),
        shader_validation: true,
    }
}
