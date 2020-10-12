use std::marker::PhantomData;

use super::frame::Frame;

pub struct Uniform<T: Copy> {
    pub buffer: wgpu::Buffer,
    data: PhantomData<T>,
}

impl<T: Copy> Uniform<T> {
    const SIZE: wgpu::BufferAddress = std::mem::size_of::<T>() as u64;

    pub fn new(device: &wgpu::Device, label: &str, initial: Option<&T>) -> Self {
        use wgpu::util::DeviceExt as _;
        let buffer = match initial {
            None => device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: Self::SIZE,
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                mapped_at_creation: false,
            }),
            Some(t) => device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: unsafe { safe_transmute::to_bytes::transmute_to_bytes_unchecked(t) },
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }),
        };

        Self {
            buffer,
            data: PhantomData,
        }
    }

    pub fn upload(&self, frame: &mut Frame, t: &T) {
        frame.write_uniform(&self, t);
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn size(&self) -> wgpu::BufferAddress {
        Self::SIZE
    }
}

pub struct UniformArray<T: Copy> {
    pub buffer: wgpu::Buffer,
    n: u64,
    data: PhantomData<T>,
}

impl<T: Copy> UniformArray<T> {
    const SIZE: wgpu::BufferAddress = std::mem::size_of::<T>() as wgpu::BufferAddress;

    pub fn new(device: &wgpu::Device, label: &str, n: usize) -> Self {
        let n = n as u64;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: Self::SIZE * n,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            n,
            data: PhantomData,
        }
    }

    pub fn upload(&self, frame: &mut Frame, ts: &[T]) {
        self.upload_slice(frame, 0, ts)
    }

    pub fn upload_slice(&self, frame: &mut Frame, i: usize, ts: &[T]) {
        frame.write_uniform_slice(&self, i, ts)
    }

    pub fn upload_el(&self, frame: &mut Frame, i: usize, t: &T) {
        frame.write_uniform_el(&self, i, t);
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn size(&self) -> wgpu::BufferAddress {
        Self::SIZE * self.n
    }

    pub fn t_size(&self) -> wgpu::BufferAddress {
        Self::SIZE
    }
}

pub struct UniformStorage<T: Copy> {
    pub v: T,
    pub uniform: Uniform<T>,
}

impl<T: Copy> UniformStorage<T> {
    pub fn new(device: &wgpu::Device, label: &'static str, v: T) -> Self {
        Self {
            v,
            uniform: Uniform::new(device, label, Some(&v)),
        }
    }

    pub fn update(&self, frame: &mut Frame) {
        self.uniform.upload(frame, &self.v);
    }
}

impl<T: Copy> AsRef<Uniform<T>> for UniformStorage<T> {
    fn as_ref(&self) -> &Uniform<T> {
        &self.uniform
    }
}

impl<T: Copy> std::ops::Deref for UniformStorage<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.v
    }
}

impl<T: Copy> std::ops::DerefMut for UniformStorage<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.v
    }
}
