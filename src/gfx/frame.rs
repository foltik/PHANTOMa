use std::sync::Arc;

use super::uniform::{Uniform, UniformArray};
use super::wgpu;

pub struct Frame {
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) queue: Arc<wgpu::Queue>,
    pub(crate) staging: wgpu_async_staging::StagingBelt,
    pub(crate) encoder: Option<wgpu::CommandEncoder>,

    // begin: Instant,
}

impl Frame {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, staging: wgpu_async_staging::StagingBelt) -> Self {
        let encoder = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        Self {
            device,
            queue,
            staging,
            encoder: Some(encoder),
            // begin: Instant::now()
        }
    }

    pub(crate) fn write_buffer<T: Copy>(
        &mut self,
        buffer: &wgpu::Buffer,
        offset: u64,
        data: &[T],
    ) {
        let bytes = unsafe { safe_transmute::to_bytes::transmute_to_bytes_many_unchecked(data) };

        let sz = std::num::NonZeroU64::new(bytes.len() as u64).unwrap();

        let mut view = self.staging.write_buffer(
            self.encoder.as_mut().unwrap(),
            buffer,
            offset,
            sz,
            &self.device,
        );
        view.copy_from_slice(bytes);
    }

    pub(crate) fn write_uniform<T: Copy>(&mut self, u: &Uniform<T>, t: &T) {
        self.write_buffer(&u.buffer, 0, std::slice::from_ref(t));
    }

    pub(crate) fn write_uniform_slice<T: Copy>(&mut self, u: &UniformArray<T>, i: usize, ts: &[T]) {
        self.write_buffer(&u.buffer, i as u64, ts)
    }

    pub(crate) fn write_uniform_el<T: Copy>(&mut self, u: &UniformArray<T>, i: usize, t: &T) {
        self.write_uniform_slice(u, i, std::slice::from_ref(t));
    }

    pub fn submit(mut self) -> wgpu_async_staging::StagingBelt {
        self.staging.finish();
        let buffer = self.encoder.take().unwrap().finish();
        // log::trace!("Frame encoded in {:?}", self.begin.elapsed());
        
        // let pre_submit = Instant::now();
        self.queue.submit(Some(buffer));
        // log::trace!("Commands submitted in {:?}", pre_submit.elapsed());

        self.staging
    }
}