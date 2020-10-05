use super::wgpu;
use super::uniform::Uniform;

pub struct Frame<'a> {
    pub device: &'a wgpu::Device,
    pub view: wgpu::SwapChainTextureView,
    pub encoder: wgpu::CommandEncoder,

    pub(crate) staging: &'a mut wgpu::util::StagingBelt,
}

impl<'a> Frame<'a> {
    pub(crate) fn new(
        device: &'a wgpu::Device,
        staging: &'a mut wgpu::util::StagingBelt,
        view: wgpu::SwapChainTextureView,
    ) -> Self {
        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("PHANTOMA_frame_encoder"),
        });


        Self {
            device, 
            view,
            encoder,

            staging,
        }
    }

    pub(crate) fn write_uniform<T: Copy>(&mut self, u: &Uniform<T>, t: &T) {
        let sz = std::num::NonZeroU64::new(u.size()).unwrap();
        let mut view = self.staging.write_buffer(&mut self.encoder, &u.buffer, 0, sz, self.device);
        let bytes = unsafe { safe_transmute::to_bytes::transmute_to_bytes_unchecked(t) };
        view.copy_from_slice(bytes);
    }

    // pub(crate) fn write_buffer(target: &wgpu::Buffer, offset: wgpu::BufferAddress, size: wgpu::BufferSize)

    pub(crate) fn submit(self, queue: &wgpu::Queue) {
        self.staging.finish();

        let buffer = self.encoder.finish();
        queue.submit(Some(buffer));

        // async_std::task::block_on(self.staging.recall());
    }
}
