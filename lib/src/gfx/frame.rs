// use std::time::Instant;

use crate::app::App;

use super::uniform::{Uniform, UniformArray};
use super::wgpu;

pub struct Frame<'a> {
    pub(crate) app: &'a App,
    pub(crate) encoder: Option<wgpu::CommandEncoder>,

    // begin: Instant,
}

impl<'a> Frame<'a> {
    pub fn new(app: &'a App) -> Self {
        let encoder = app
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        Self {
            app,
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

        let mut staging = self.app.staging.borrow_mut();
        let mut view = staging.write_buffer(
            self.encoder.as_mut().unwrap(),
            buffer,
            offset,
            sz,
            &self.app.device,
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

    pub fn submit(&mut self) {
        self.app.staging.borrow_mut().finish();
        let buffer = self.encoder.take().unwrap().finish();
        // log::trace!("Frame encoded in {:?}", self.begin.elapsed());
        
        // let pre_submit = Instant::now();
        self.app.queue.submit(Some(buffer));
        // log::trace!("Commands submitted in {:?}", pre_submit.elapsed());
    }
}