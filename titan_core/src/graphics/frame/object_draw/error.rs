use thiserror::Error;
use vulkano::command_buffer::{BuildError, DrawIndexedError};
use vulkano::memory::DeviceMemoryAllocError;
use vulkano::pipeline::GraphicsPipelineCreationError;
use vulkano::sync::FlushError;
use vulkano::OomError;

use crate::graphics::renderer::error::DescriptorSetCreationError;

#[derive(Debug, Error)]
pub enum ObjectDrawSystemCreationError {
    #[error("shader module allocation failure: {0}")]
    OutOfMemory(#[from] OomError),

    #[error("queue family must support graphics operations")]
    QueueFamilyNotSupported,

    #[error("graphics pipeline creation failure: {0}")]
    GraphicsPipelineCreation(#[from] GraphicsPipelineCreationError),

    #[error("vertex/index buffer creation failure: {0}")]
    BufferCreation(#[from] FlushError),

    #[error("vertex/index buffer allocation failure: {0}")]
    BufferAllocation(#[from] DeviceMemoryAllocError),
}

#[derive(Debug, Error)]
pub enum ObjectDrawError {
    #[error("command buffer allocation failure: {0}")]
    OutOfMemory(#[from] OomError),

    #[error("draw indexed command failure: {0}")]
    DrawIndexed(#[from] DrawIndexedError),

    #[error("uniform buffer descriptor set creation failure: {0}")]
    DescriptorSetCreation(#[from] DescriptorSetCreationError),

    #[error("draw command buffer build failure: {0}")]
    CommandBufferBuild(#[from] BuildError),
}
