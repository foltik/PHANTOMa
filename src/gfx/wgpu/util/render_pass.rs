// FIXME: Add label param to render pass

pub struct OpsBuilder<T> {
    ops: wgpu::Operations<T>,
}

impl<T> OpsBuilder<T> {
    fn new(ops: wgpu::Operations<T>) -> Self {
        Self { ops }
    }

    pub fn load(mut self) -> Self {
        self.ops.load = wgpu::LoadOp::Load;
        self
    }

    pub fn clear(mut self, t: T) -> Self {
        self.ops.load = wgpu::LoadOp::Clear(t);
        self
    }

    pub fn store(mut self) -> Self {
        self.ops.store = true;
        self
    }

    pub fn discard(mut self) -> Self {
        self.ops.store = false;
        self
    }
}

/// A builder type to simplify the process of creating a render pass descriptor.
#[derive(Debug)]
pub struct ColorAttachmentDescriptorBuilder<'a> {
    descriptor: wgpu::RenderPassColorAttachment<'a>,
}

impl<'a> ColorAttachmentDescriptorBuilder<'a> {
    pub const DEFAULT_COLOR_OPS: wgpu::Operations<wgpu::Color> = wgpu::Operations {
        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        store: true,
    };

    /// Begin building a new render pass color attachment descriptor.
    fn new(view: &'a wgpu::TextureView) -> Self {
        Self {
            descriptor: wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Self::DEFAULT_COLOR_OPS,
            },
        }
    }

    /// Specify the resolve target for this render pass color attachment.
    pub fn resolve_target(mut self, target: Option<&'a wgpu::TextureView>) -> Self {
        self.descriptor.resolve_target = target;
        self
    }

    pub fn color<F: FnOnce(OpsBuilder<wgpu::Color>) -> OpsBuilder<wgpu::Color>>(mut self, builder: F) -> Self {
        self.descriptor.ops = builder(OpsBuilder::new(self.descriptor.ops)).ops;
        self
    }
}


/// A builder type to simplify the process of creating a render pass descriptor.
#[derive(Debug)]
pub struct DepthStencilAttachmentDescriptorBuilder<'a> {
    descriptor: wgpu::RenderPassDepthStencilAttachment<'a>,
}

impl<'a> DepthStencilAttachmentDescriptorBuilder<'a> {
    pub const DEFAULT_DEPTH_OPS: wgpu::Operations<f32> = wgpu::Operations {
        load: wgpu::LoadOp::Clear(1.0),
        store: true,
    };

    pub const DEFAULT_STENCIL_OPS: wgpu::Operations<u32> = wgpu::Operations {
        load: wgpu::LoadOp::Clear(0),
        store: true,
    };

    fn new(view: &'a wgpu::TextureView) -> Self {
        DepthStencilAttachmentDescriptorBuilder {
            descriptor: wgpu::RenderPassDepthStencilAttachment {
                view,
                depth_ops: Some(Self::DEFAULT_DEPTH_OPS),
                stencil_ops: None,
            },
        }
    }

    pub fn depth<F: FnOnce(OpsBuilder<f32>) -> OpsBuilder<f32>>(mut self, builder: F) -> Self {
        self.descriptor.depth_ops = Some(builder(OpsBuilder::new(Self::DEFAULT_DEPTH_OPS)).ops);
        self
    }

    pub fn stencil<F: FnOnce(OpsBuilder<u32>) -> OpsBuilder<u32>>(mut self, builder: F) -> Self {
        self.descriptor.stencil_ops = Some(builder(OpsBuilder::new(Self::DEFAULT_STENCIL_OPS)).ops);
        self
    }
}


/// A builder type to simplify the process of creating a render pass descriptor.
#[derive(Debug, Default)]
pub struct Builder<'a> {
    color_attachments: Vec<wgpu::RenderPassColorAttachment<'a>>,
    depth_stencil_attachment:
        Option<wgpu::RenderPassDepthStencilAttachment<'a>>,
}

impl<'a> Builder<'a> {
    /// Begin building a new render pass descriptor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single color attachment descriptor to the render pass descriptor.
    ///
    /// Call this multiple times in succession to add multiple color attachments.
    pub fn color_attachment<F>(
        mut self,
        attachment: &'a wgpu::TextureView,
        color_builder: F,
    ) -> Self
    where
        F: FnOnce(ColorAttachmentDescriptorBuilder<'a>) -> ColorAttachmentDescriptorBuilder<'a>,
    {
        let builder = ColorAttachmentDescriptorBuilder::new(attachment);
        let descriptor = color_builder(builder).descriptor;
        self.color_attachments.push(descriptor);
        self
    }

    /// Add a depth stencil attachment to the render pass.
    ///
    /// This should only be called once, as only a single depth stencil attachment is valid. Only
    /// the attachment submitted last will be used.
    pub fn depth_stencil_attachment<F>(
        mut self,
        attachment: &'a wgpu::TextureView,
        depth_stencil_builder: F,
    ) -> Self
    where
        F: FnOnce(
            DepthStencilAttachmentDescriptorBuilder<'a>,
        ) -> DepthStencilAttachmentDescriptorBuilder<'a>,
    {
        let builder = DepthStencilAttachmentDescriptorBuilder::new(attachment);
        let descriptor = depth_stencil_builder(builder).descriptor;
        self.depth_stencil_attachment = Some(descriptor);
        self
    }

    /// Begin a render pass with the specified parameters on the given encoder.
    pub fn begin_encoder(self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::RenderPass {
        let Builder {
            color_attachments,
            depth_stencil_attachment
        } = self;

        let descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment,
        };

        encoder.begin_render_pass(&descriptor)
    }

    pub fn begin(self, frame: &'a mut crate::gfx::frame::Frame) -> wgpu::RenderPass<'a> {
        self.begin_encoder(frame.encoder.as_mut().unwrap())
    }
}
