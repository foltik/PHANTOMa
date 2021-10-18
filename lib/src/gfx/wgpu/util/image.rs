use crate::app::App;
use crate::gfx::wgpu;

use crate::math::Vector2;
use image::{DynamicImage, RgbaImage};

use std::num::NonZeroU32;

fn nearest(n: u32) -> u32 {
    std::cmp::max(64, 2u32.pow((n as f32).log2().ceil() as u32))
}

pub fn resize(image: RgbaImage) -> (RgbaImage, Vector2) {
    let (ow, oh) = image.dimensions();
    let (w, h) = (nearest(ow), nearest(oh));

    if ow != w || oh != h {
        let scale = Vector2::new(ow as f32 / w as f32, oh as f32 / h as f32);

        let mut resized = RgbaImage::new(w, h);
        for x in 0..w {
            for y in 0..h {
                resized.put_pixel(x, y, *image.get_pixel(x % ow, y % oh));
            }
        }

        // log::trace!("Resized image from {}x{} to {}x{}", ow, oh, w, h);

        (resized, scale)
    } else {
        (image, Vector2::new(1.0, 1.0))
    }
}

pub fn load(app: &App, image: &DynamicImage) -> (wgpu::Texture, Vector2) {
    let (image, scale) = resize(image.to_rgba());
    let (width, height) = image.dimensions();

    let sz = width * height * 4;
    let extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = wgpu::util::TextureBuilder::new("image")
        .format(wgpu::defaults::texture_format())
        .size([width, height, 1])
        .usage(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT)
        .build(&app.device);

    let buffer = app.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: sz as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: true,
    });

    let raw = image.into_raw();

    buffer
        .slice(..)
        .get_mapped_range_mut()
        .copy_from_slice(&raw);

    buffer.unmap();

    let buffer_copy_view = wgpu::ImageCopyBuffer {
        buffer: &buffer,
        layout: wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(width * 4),
            rows_per_image: NonZeroU32::new(height),
        },
    };

    let texture_copy_view = wgpu::ImageCopyTexture {
        texture: &texture,
        aspect: wgpu::TextureAspect::All,
        origin: wgpu::Origin3d::ZERO,
        mip_level: 0,
    };

    let mut encoder = app.encoder("image_upload");
    encoder.copy_buffer_to_texture(buffer_copy_view, texture_copy_view, extent);
    app.submit(encoder);

    (texture, scale)
}

pub fn load_array(app: &App, images: &[DynamicImage]) -> (wgpu::Texture, Vector2) {
    let n = images.len();

    let images = images.iter().map(|i| resize(i.to_rgba())).collect::<Vec<_>>();
    let scale = images[0].1;
    let (width, height) = images[0].0.dimensions();

    let sz: usize = (width * height * 4) as usize;
    let extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: n as u32,
    };

    let texture = wgpu::util::TextureBuilder::new("image")
        .format(wgpu::defaults::texture_format())
        .size([width, height, n as u32])
        .usage(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT)
        .build(&app.device);

    let buffer = app.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: (width * height * 4 * n as u32) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: true,
    });

    {
        let mut view = buffer.slice(..).get_mapped_range_mut();

        let mut pos = 0;
        for i in images {
            view[pos..(pos + sz)].copy_from_slice(&i.0.into_raw());
            pos += sz;
        }
    }

    buffer.unmap();

    let buffer_copy_view = wgpu::ImageCopyBuffer {
        buffer: &buffer,
        layout: wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(width * 4),
            rows_per_image: NonZeroU32::new(height),
        },
    };

    let texture_copy_view = wgpu::ImageCopyTexture {
        texture: &texture,
        aspect: wgpu::TextureAspect::All,
        origin: wgpu::Origin3d::ZERO,
        mip_level: 0,
    };

    let mut encoder = app.encoder("image_upload");
    encoder.copy_buffer_to_texture(buffer_copy_view, texture_copy_view, extent);
    app.submit(encoder);

    (texture, scale)
}
