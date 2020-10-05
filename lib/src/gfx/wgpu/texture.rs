use std::ops::Deref;

use crate::window::SwapChain;
use crate::gfx::wgpu::util::TextureViewBuilder;

pub struct Texture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) label: String,
    pub(crate) size: wgpu::Extent3d,
    pub(crate) dimension: wgpu::TextureDimension,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) mip_count: u32,
    pub(crate) sample_count: u32,
}

impl Texture {
    pub fn view<'a>(&'a self) -> TextureViewBuilder<'a> {
        TextureViewBuilder::new(&self)
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn size(&self) -> &wgpu::Extent3d {
        &self.size
    }

    pub fn dimension(&self) -> &wgpu::TextureDimension {
        &self.dimension
    }

    pub fn format(&self) -> &wgpu::TextureFormat {
        &self.format
    }

    pub fn mip_level_count(&self) -> u32 {
        self.mip_count
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }
}

impl Deref for Texture {
    type Target = wgpu::Texture;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

pub struct TextureView {
    pub(crate) view: wgpu::TextureView,
    pub(crate) size: wgpu::Extent3d,
    pub(crate) dimension: wgpu::TextureViewDimension,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) mip_count: u32,
    pub(crate) sample_count: u32,
}

impl TextureView {
    pub fn size(&self) -> &wgpu::Extent3d {
        &self.size
    }

    pub fn dimension(&self) -> &wgpu::TextureViewDimension {
        &self.dimension
    }

    pub fn format(&self) -> &wgpu::TextureFormat {
        &self.format
    }

    pub fn mip_level_count(&self) -> u32 {
        self.mip_count
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }
}

impl Deref for TextureView {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

pub struct SwapChainTextureView {
    frame: wgpu::SwapChainFrame,
    pub size: wgpu::Extent3d,
    pub dimension: wgpu::TextureViewDimension,
    pub format: wgpu::TextureFormat,
}

impl SwapChainTextureView {
    pub(crate) fn new(swap_chain: &SwapChain, frame: wgpu::SwapChainFrame) -> Self {
        Self {
            frame,
            size: wgpu::Extent3d { 
                width: swap_chain.descriptor.width,
                height: swap_chain.descriptor.height,
                depth: 1,
            },
            dimension: wgpu::TextureViewDimension::D2,
            format: swap_chain.descriptor.format,
        }
    }
}

impl Deref for SwapChainTextureView {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        &self.frame.output.view
    }
}


pub trait View {
    fn view(&self) -> &wgpu::TextureView;
}

impl<T: Deref<Target = wgpu::TextureView>> View for T {
    fn view(&self) -> &wgpu::TextureView {
        self.deref()
    }
}