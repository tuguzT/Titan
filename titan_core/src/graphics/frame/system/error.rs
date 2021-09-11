use thiserror::Error;
use vulkano::command_buffer::{
    AutoCommandBufferBuilderContextError, BeginRenderPassError, BuildError, CommandBufferExecError,
    ExecuteCommandsError,
};
use vulkano::image::view::ImageViewCreationError;
use vulkano::image::ImageCreationError;
use vulkano::render_pass::{FramebufferCreationError, RenderPassCreationError};
use vulkano::OomError;

#[derive(Debug, Error)]
pub enum FrameSystemCreationError {
    #[error("queue family must support graphics operations")]
    QueueFamilyNotSupported,

    #[error("render pass creation failure: {0}")]
    RenderPassCreation(#[from] RenderPassCreationError),
}

#[derive(Debug, Error)]
pub enum FrameCreationError {
    #[error("frame command buffer allocation failure: {0}")]
    OutOfMemory(#[from] OomError),

    #[error("begin render pass command failure: {0}")]
    BeginRenderPass(#[from] BeginRenderPassError),

    #[error("failed to recreate an image for the frame: {0}")]
    ImageCreation(#[from] ImageCreationError),

    #[error("failed to create an image view for the frame: {0}")]
    ImageViewCreation(#[from] ImageViewCreationError),

    #[error("failed to create framebuffer for the frame: {0}")]
    FramebufferCreation(#[from] FramebufferCreationError),
}

#[derive(Debug, Error)]
pub enum NextPassError {
    #[error("next pass command buffer building error: {0}")]
    WrongUsage(#[from] AutoCommandBufferBuilderContextError),

    #[error("next pass command buffer build failure: {0}")]
    Build(#[from] BuildError),

    #[error("next pass command buffer execution failure: {0}")]
    Execution(#[from] CommandBufferExecError),
}

#[derive(Debug, Error)]
pub enum DrawPassExecuteError {
    #[error("draw pass secondary command buffer execution failure: {0}")]
    Execution(#[from] ExecuteCommandsError),
}
