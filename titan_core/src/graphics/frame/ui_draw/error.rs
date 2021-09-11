use thiserror::Error;
use vulkano::command_buffer::{BuildError, DrawIndexedError};
use vulkano::image::view::ImageViewCreationError;
use vulkano::image::ImageCreationError;
use vulkano::memory::DeviceMemoryAllocError;
use vulkano::pipeline::GraphicsPipelineCreationError;
use vulkano::sampler::SamplerCreationError;
use vulkano::sync::FlushError;
use vulkano::OomError;

use crate::graphics::renderer::error::DescriptorSetCreationError;

#[derive(Debug, Error)]
pub enum UiDrawSystemCreationError {
    #[error("shader module allocation failure: {0}")]
    OutOfMemory(#[from] OomError),

    #[error("queue family must support graphics operations")]
    QueueFamilyNotSupported,

    #[error("graphics pipeline creation failure: {0}")]
    GraphicsPipelineCreation(#[from] GraphicsPipelineCreationError),

    #[error("texture sampler creation failure: {0}")]
    SamplerCreation(#[from] SamplerCreationError),
}

#[derive(Debug, Error)]
pub enum UiDrawError {
    #[error("command buffer allocation failure: {0}")]
    OutOfMemory(#[from] OomError),

    #[error("sampled texture creation failure: {0}")]
    ImageCreation(#[from] ImageCreationError),

    #[error("sampled texture creation failure on waiting: {0}")]
    WaitOnImageCreation(#[from] FlushError),

    #[error("sampled texture view creation failure: {0}")]
    ImageViewCreation(#[from] ImageViewCreationError),

    #[error("sampled image descriptor set creation failure: {0}")]
    DescriptorSetCreation(#[from] DescriptorSetCreationError),

    #[error("vertex/index buffer allocation failure: {0}")]
    BufferAllocation(#[from] DeviceMemoryAllocError),

    #[error("draw indexed command failure: {0}")]
    DrawIndexed(#[from] DrawIndexedError),

    #[error("draw command buffer build failure: {0}")]
    CommandBufferBuild(#[from] BuildError),
}
