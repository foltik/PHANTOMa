use crate::app::App;
use super::wgpu;
use super::uniform::{Uniform, UniformArray};

pub struct Frame<'a> {
    app: &'a mut App,
    pub(crate) encoder: Option<wgpu::CommandEncoder>,

    // pub(crate) staging: &'a mut wgpu::util::StagingBelt,
}

impl<'a> Frame<'a> {
    pub fn new(
        app: &'a mut App,
        // staging: &'a mut wgpu::util::StagingBelt,
        // view: wgpu::SwapChainTextureView,
    ) -> Self {
        let encoder = app.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame_encoder"),
        });

        Self {
            app,
            // view,
            encoder: Some(encoder),

            // staging,
        }
    }

    pub(crate) fn write_buffer<T: Copy>(
        &mut self,
        buffer: &wgpu::Buffer,
        offset: u64,
        sz: u64,
        data: &[T],
    ) {
        let sz = std::num::NonZeroU64::new(sz).unwrap();
        let mut view =
            self.app.staging
                .write_buffer(self.encoder.as_mut().unwrap(), buffer, offset, sz, &self.app.device);

        let bytes = unsafe { safe_transmute::to_bytes::transmute_to_bytes_many_unchecked(data) };
        view.copy_from_slice(bytes);
    }

    pub(crate) fn write_uniform<T: Copy>(&mut self, u: &Uniform<T>, t: &T) {
        self.write_buffer(&u.buffer, 0, u.size(), std::slice::from_ref(t));
    }

    pub(crate) fn write_uniform_slice<T: Copy>(&mut self, u: &UniformArray<T>, i: usize, ts: &[T]) {
        self.write_buffer(&u.buffer, i as u64, u.size(), ts)
    }

    pub(crate) fn write_uniform_el<T: Copy>(&mut self, u: &UniformArray<T>, i: usize, t: &T) {
        self.write_uniform_slice(u, i, std::slice::from_ref(t));
    }

    pub fn submit(&mut self) {
        self.app.staging.finish();

        let buffer = self.encoder.take().unwrap().finish();
        self.app.queue.submit(Some(buffer));
    }
}

impl<'a> std::ops::Drop for Frame<'a> {
    fn drop(&mut self) {
        if self.encoder.is_some() {
            panic!("Frame dropped without submitting!");
        }
    }
}