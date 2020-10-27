use std::marker::PhantomData;

use super::frame::Frame;

#[derive(Debug)]
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

    pub fn new(device: &wgpu::Device, label: &str, n: usize, initial: Option<&[T]>) -> Self {
        let n = n as u64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: Self::SIZE * n,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: true,
        });

        if let Some(ts) = initial {
            let bytes = unsafe { safe_transmute::to_bytes::transmute_to_bytes_many_unchecked(ts) };

            buffer
                .slice(0..bytes.len() as wgpu::BufferAddress)
                .get_mapped_range_mut()
                .copy_from_slice(bytes);
        }

        buffer.unmap();

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

#[derive(Debug)]
pub struct UniformStorage<T: Copy> {
    pub v: T,
    pub uniform: Uniform<T>,
}

impl<T: Copy> UniformStorage<T> {
    pub fn new(device: &wgpu::Device, label: &str, v: T) -> Self {
        Self {
            v,
            uniform: Uniform::new(device, label, Some(&v)),
        }
    }

    pub fn upload(&self, frame: &mut Frame) {
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


pub struct UniformArrayStorage<T: Copy> {
    pub vs: Vec<T>,
    pub uniform: UniformArray<T>,
}

impl<T: Copy> UniformArrayStorage<T> {
    pub fn new(device: &wgpu::Device, label: &str, n: usize, vs: Vec<T>) -> Self {
        let uniform = UniformArray::new(device, label, n, Some(&vs));
        Self {
            vs,
            uniform,
        }
    }

    pub fn upload(&self, frame: &mut Frame) {
        self.uniform.upload(frame, &self.vs)
    }

    pub fn upload_slice(&self, frame: &mut Frame, offset: usize, i: usize, j: usize) {
        self.uniform.upload_slice(frame, offset, &self.vs[i..j])
    }

    pub fn upload_el(&self, frame: &mut Frame, offset: usize, i: usize) {
        self.uniform.upload_el(frame, offset, &self.vs[i])
    }
}

impl<T: Copy> AsRef<UniformArray<T>> for UniformArrayStorage<T> {
    fn as_ref(&self) -> &UniformArray<T> {
        &self.uniform
    }
}

impl<T: Copy> std::ops::Deref for UniformArrayStorage<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.vs
    }
}

impl<T: Copy> std::ops::DerefMut for UniformArrayStorage<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vs
    }
}