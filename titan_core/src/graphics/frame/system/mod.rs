use std::sync::Arc;

use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, SecondaryCommandBuffer,
    SubpassContents,
};
use vulkano::device::Queue;
use vulkano::format::{ClearValue, Format};
use vulkano::image::view::ImageView;
use vulkano::image::{AttachmentImage, ImageAccess, ImageUsage};
use vulkano::render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass};
use vulkano::sync::GpuFuture;

use error::{DrawPassExecuteError, FrameCreationError, FrameSystemCreationError, NextPassError};

use crate::{graphics::utils, window::Size};

pub mod error;

/// System that contains the necessary facilities for rendering a single frame.
pub struct FrameSystem {
    /// Queue to render everything.
    graphics_queue: Arc<Queue>,

    /// Render pass used for the drawing.
    render_pass: Arc<RenderPass>,

    /// Intermediate render target that will contain the depth of each pixel of the scene.
    /// This is a traditional depth buffer. `0.0` means "near", and `1.0` means "far".
    depth_buffer: Option<Arc<AttachmentImage>>,
}

impl FrameSystem {
    /// Creates the frame system.
    pub fn new(
        graphics_queue: Arc<Queue>,
        final_output_format: Format,
    ) -> Result<Self, FrameSystemCreationError> {
        // Check queue for graphics support.
        if !graphics_queue.family().supports_graphics() {
            return Err(FrameSystemCreationError::QueueFamilyNotSupported);
        }

        let device = graphics_queue.device().clone();
        let depth_format = utils::suitable_depth_stencil_format(device.physical_device());

        // TODO: vulkano error: https://github.com/vulkano-rs/vulkano/issues/1665
        let render_pass = Arc::new(vulkano::ordered_passes_renderpass! {
            graphics_queue.device().clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: final_output_format,
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: depth_format,
                    samples: 1,
                    initial_layout: ImageLayout::Undefined,
                    final_layout: ImageLayout::DepthStencilAttachmentOptimal,
                }
            },
            passes: [
                // Subpass for complex rendering.
                { color: [color], depth_stencil: {depth}, input: [] },
                // Subpass for UI rendering.
                { color: [color], depth_stencil: {}, input: [] }
            ]
        }?);

        Ok(Self {
            graphics_queue,
            render_pass,
            depth_buffer: None,
        })
    }

    /// Retrieve subpass for object rendering.
    pub fn object_subpass(&self) -> Subpass {
        Subpass::from(self.render_pass.clone(), 0).unwrap()
    }

    /// Retrieve subpass for UI rendering.
    pub fn ui_subpass(&self) -> Subpass {
        Subpass::from(self.render_pass.clone(), 1).unwrap()
    }

    /// Starts drawing a new frame.
    pub fn frame<F, I>(
        &mut self,
        before_future: F,
        final_image: Arc<I>,
    ) -> Result<Frame, FrameCreationError>
    where
        F: GpuFuture + Send + Sync + 'static,
        I: ImageAccess + Send + Sync + 'static,
    {
        let device = self.graphics_queue.device().clone();

        let dimensions = final_image.dimensions().width_height();
        let old_dimensions = self
            .depth_buffer
            .as_ref()
            .map(|b| b.dimensions().width_height());

        // If there is no depth buffer (first call after initialization)
        // or dimensions are incompatible, (re)create buffers.
        if old_dimensions.is_none() || old_dimensions.unwrap() != dimensions {
            // (Re)create depth buffer.
            let depth_buffer = {
                let depth_format = utils::suitable_depth_stencil_format(device.physical_device());
                AttachmentImage::with_usage(
                    device.clone(),
                    dimensions,
                    depth_format,
                    ImageUsage::depth_stencil_attachment(),
                )?
            };
            self.depth_buffer = Some(depth_buffer.clone());
        }

        // Create framebuffer.
        let framebuffer = {
            let image_view = ImageView::new(final_image.clone())?;
            let depth_buffer_view = {
                let depth_buffer = self.depth_buffer.as_ref().unwrap().clone();
                ImageView::new(depth_buffer)?
            };
            Arc::new(
                Framebuffer::start(self.render_pass.clone())
                    .add(image_view)?
                    .add(depth_buffer_view)?
                    .build()?,
            )
        };

        let clear_values = [
            ClearValue::Float([0.0, 0.0, 0.0, 1.0]),
            ClearValue::Depth(1.0),
        ];

        // Build primary command buffer that will execute secondary command buffers
        // in rendering process.
        let mut builder = AutoCommandBufferBuilder::primary(
            device,
            self.graphics_queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )?;
        builder.begin_render_pass(
            framebuffer.clone(),
            SubpassContents::SecondaryCommandBuffers,
            clear_values,
        )?;

        Ok(Frame {
            system: self,
            subpass_number: 0,
            before_future: Some(Box::new(before_future)),
            framebuffer,
            command_buffer_builder: Some(builder),
        })
    }
}

/// Represents the active process of rendering a frame.
pub struct Frame<'a> {
    /// The borrowed `FrameSystem`.
    system: &'a mut FrameSystem,

    /// The active pass we are in. This keeps track of the step we are in.
    subpass_number: u8,

    /// Future to wait upon before the main rendering.
    before_future: Option<Box<dyn GpuFuture + Send + Sync>>,

    /// Framebuffer that was used when starting the render pass.
    framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,

    /// The command buffer builder that will be built during the lifetime of this object.
    command_buffer_builder: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
}

impl<'a> Frame<'a> {
    /// Returns an enumeration containing the next pass of the rendering.
    pub fn next_pass<'f>(&'f mut self) -> Result<Option<Pass<'f, 'a>>, NextPassError> {
        match {
            let current_pass = self.subpass_number;
            self.subpass_number += 1;
            current_pass
        } {
            // If we are in the pass 0 then we haven't start anything yet.
            // We return an object that will allow the user to draw objects on the scene.
            0 => Ok(Some(Pass::Deferred(DrawPass { frame: self }))),

            // If we are in the pass 1 then we have finished drawing the objects on the scene.
            1 => {
                self.command_buffer_builder
                    .as_mut()
                    .unwrap()
                    .next_subpass(SubpassContents::SecondaryCommandBuffers)?;

                // Returning an object that will allow the user to render UI.
                Ok(Some(Pass::UI(DrawPass { frame: self })))
            }

            // If we are in pass 2 then we have finished rendering UI.
            2 => {
                self.command_buffer_builder
                    .as_mut()
                    .unwrap()
                    .end_render_pass()?;
                let command_buffer = self.command_buffer_builder.take().unwrap().build()?;

                // Extract `before_future` and append the command buffer execution to it.
                let after_future = self
                    .before_future
                    .take()
                    .unwrap()
                    .then_execute(self.system.graphics_queue.clone(), command_buffer)?;

                // We obtain `after_future`, which we give to the user.
                Ok(Some(Pass::Finished(Box::new(after_future))))
            }

            // The frame is in the finished state and we can't do anything.
            _ => Ok(None),
        }
    }
}

/// Struct provided to the user that allows them to customize or handle the pass.
pub enum Pass<'f, 's: 'f> {
    /// We are in the pass where we draw objects on the scene.
    /// The `DrawPass` allows the user to draw the objects.
    Deferred(DrawPass<'f, 's>),

    /// We are in the pass where we draw UI on the screen.
    /// The `DrawPass` allows the user to draw the UI.
    UI(DrawPass<'f, 's>),

    /// The frame has been fully prepared, and here is the future that will perform the drawing
    /// on the image.
    Finished(Box<dyn GpuFuture + Send + Sync>),
}

/// Allows the user to draw objects on the scene.
pub struct DrawPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> DrawPass<'f, 's> {
    /// Appends a command that executes a secondary command buffer that performs drawing.
    pub fn execute<C>(&mut self, secondary_command_buffer: C) -> Result<(), DrawPassExecuteError>
    where
        C: SecondaryCommandBuffer + Send + Sync + 'static,
    {
        self.frame
            .command_buffer_builder
            .as_mut()
            .unwrap()
            .execute_commands(secondary_command_buffer)?;
        Ok(())
    }

    /// Returns the dimensions in pixels of the viewport.
    pub fn viewport_size(&self) -> Size {
        let dimensions = self.frame.framebuffer.dimensions();
        Size::new(dimensions[0], dimensions[1])
    }
}
