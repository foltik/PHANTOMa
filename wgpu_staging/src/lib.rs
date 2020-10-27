#![feature(async_closure)]

use futures::future::join_all;

struct Chunk {
    buffer: wgpu::Buffer,
    size: u64,
    offset: u64,
}

/// Staging pool is a machine that uploads data.
///
/// Internally it uses a ring-buffer of staging buffers that are sub-allocated.
/// It has an advantage over `Queue.write_buffer` in a way that it returns a mutable slice,
/// which you can fill to avoid an extra data copy.
///
/// Using a staging pool is slightly complicated, and generally goes as follows:
/// - Write to buffers that need writing to using `write_buffer`.
/// - Call `finish`.
/// - Submit all command encoders used with `write_buffer`.
/// - Call `recall`
pub struct StagingPool {
    chunk_size: u64,
    /// Chunks that we are actively using for pending transfers at this moment.
    active_chunks: Vec<Chunk>,
    /// Chunks that have scheduled transfers already.
    closed_chunks: Vec<Chunk>,
    /// Chunks that are back from the GPU and ready to be used.
    free_chunks: Vec<Chunk>,
    sender: async_channel::Sender<Chunk>,
    receiver: async_channel::Receiver<Chunk>,
}

impl StagingPool {
    /// Create a new staging pool.
    ///
    /// The `chunk_size` is the unit of internal buffer allocation.
    /// It's better when it's big, but ideally still 1-4 times less than
    /// the total amount of data uploaded per submission.
    pub fn new(chunk_size: wgpu::BufferAddress) -> Self {
        let (sender, receiver) = async_channel::unbounded();
        StagingPool {
            chunk_size,
            active_chunks: Vec::new(),
            closed_chunks: Vec::new(),
            free_chunks: Vec::new(),
            sender,
            receiver,
        }
    }

    /// Allocate the staging pool slice of `size` to be uploaded into the `target` buffer
    /// at the specified offset.
    ///
    /// The upload will be placed into the provided command encoder. This encoder
    /// must be submitted after `finish` is called and before `recall` is called.
    pub fn write_buffer(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferSize,
        device: &wgpu::Device,
    ) -> wgpu::BufferViewMut {
        let mut chunk = if let Some(index) = self
            .active_chunks
            .iter()
            .position(|chunk| chunk.offset + size.get() <= chunk.size)
        {
            self.active_chunks.swap_remove(index)
        } else if let Some(index) = self
            .free_chunks
            .iter()
            .position(|chunk| size.get() <= chunk.size)
        {
            self.free_chunks.swap_remove(index)
        } else {
            let size = self.chunk_size.max(size.get());
            // log::trace!("Allocating new staging buffer with size {}", size);
            Chunk {
                buffer: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("staging"),
                    size,
                    usage: wgpu::BufferUsage::MAP_WRITE | wgpu::BufferUsage::COPY_SRC,
                    mapped_at_creation: true,
                }),
                size,
                offset: 0,
            }
        };

        encoder.copy_buffer_to_buffer(&chunk.buffer, chunk.offset, target, offset, size.get());
        let old_offset = chunk.offset;
        chunk.offset += size.get();
        let remainder = chunk.offset % wgpu::COPY_BUFFER_ALIGNMENT;
        if remainder != 0 {
            chunk.offset += wgpu::COPY_BUFFER_ALIGNMENT - remainder;
        }

        self.active_chunks.push(chunk);
        self.active_chunks
            .last()
            .unwrap()
            .buffer
            .slice(old_offset..old_offset + size.get())
            .get_mapped_range_mut()
    }

    /// Prepare currently mapped buffers for use in a submission.
    ///
    /// At this point, all the partially used staging buffers are closed until
    /// the GPU is done copying the data from them.
    pub fn finish(&mut self) {
        for chunk in self.active_chunks.drain(..) {
            chunk.buffer.unmap();
            self.closed_chunks.push(chunk);
        }
    }

    /// Recall all of the closed buffers back to be reused.
    ///
    /// This has to be called after the command encoders written to `write_buffer` are submitted!
    pub async fn recall(&mut self) {
        while let Ok(mut chunk) = self.receiver.try_recv() {
            chunk.offset = 0;
            self.free_chunks.push(chunk);
        }

        let sender_template = &self.sender;

        join_all(self.closed_chunks.drain(..).map(async move |chunk| {
            let sender = sender_template.clone();
            chunk
                .buffer
                .slice(..)
                .map_async(wgpu::MapMode::Write).await.unwrap();
            sender.send(chunk).await.unwrap();
        })).await;
    }
}