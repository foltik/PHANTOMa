use std::marker::PhantomData;

use super::frame::Frame;

pub struct Uniform<T: Copy + Clone + 'static> {
    pub buffer: wgpu::Buffer,
    pub staging: wgpu::Buffer,
    data: PhantomData<T>,
}

impl<T: Copy + Clone + 'static> Uniform<T> {
    const SIZE: wgpu::BufferAddress = std::mem::size_of::<T>() as wgpu::BufferAddress;

    pub fn new(device: &wgpu::Device, label: &'static str) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: Self::SIZE,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: Self::SIZE,
            usage: wgpu::BufferUsage::COPY_SRC,
            mapped_at_creation: true,
        });

        Self {
            buffer,
            staging,
            data: PhantomData,
        }
    }

    // pub fn new_array(device: &wgpu::Device, n: usize) -> Self {
    //     let buffer = device.create_buffer(&wgpu::BufferDescriptor {
    //         size: Self::SIZE * n as wgpu::BufferAddress,
    //         usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    //     });

    //     Self {
    //         buffer,
    //         data: PhantomData,
    //     }
    // }

    pub fn upload(&self, frame: &mut Frame, t: &T) {
        frame.write_uniform(&self, t);
    }

    // pub fn upload_slice(
    //     &self,
    //     device: &wgpu::Device,
    //     encoder: &mut wgpu::CommandEncoder,
    //     ts: &[T],
    // ) {
    //     let n = ts.len();
    //     let staging = device
    //         .create_buffer_mapped(n, wgpu::BufferUsage::COPY_SRC)
    //         .fill_from_slice(ts);

    //     encoder.copy_buffer_to_buffer(
    //         &staging,
    //         0,
    //         &self.buffer,
    //         0,
    //         Self::SIZE * n as wgpu::BufferAddress,
    //     );
    // }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn size(&self) -> wgpu::BufferAddress {
        Self::SIZE
    }
}

pub struct UniformStorage<T: Copy + Clone + 'static> {
    pub v: T,
    pub uniform: Uniform<T>,
}

impl<T: Copy + Clone + 'static> UniformStorage<T> {
    pub fn new(device: &wgpu::Device, label: &'static str, v: T) -> Self {
        Self {
            v,
            uniform: Uniform::new(device, label),
        }
    }

    pub fn update(&self, frame: &mut Frame) {
        self.uniform.upload(frame, &self.v);
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        self.uniform.buffer()
    }
}