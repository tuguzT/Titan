use thiserror::Error;
use vulkano::command_buffer::{
    AutoCommandBufferBuilderContextError, BeginRenderPassError, BuildError, CommandBufferExecError,
    DrawIndexedError, UpdateBufferError,
};
use vulkano::descriptor_set::{PersistentDescriptorSetBuildError, PersistentDescriptorSetError};
use vulkano::device::DeviceCreationError;
use vulkano::image::view::ImageViewCreationError;
use vulkano::image::ImageCreationError;
use vulkano::instance::debug::DebugCallbackCreationError;
use vulkano::instance::InstanceCreationError;
use vulkano::memory::DeviceMemoryAllocError;
use vulkano::pipeline::GraphicsPipelineCreationError;
use vulkano::render_pass::{FramebufferCreationError, RenderPassCreationError};
use vulkano::sampler::SamplerCreationError;
use vulkano::swapchain::{AcquireError, CapabilitiesError, SwapchainCreationError};
use vulkano::sync::FlushError;
use vulkano::OomError;

#[derive(Debug, Error)]
pub enum RendererCreationError {
    #[error("failed to allocate memory for shader modules: {0}")]
    OomError(#[from] OomError),

    #[error("instance creation failure: {0}")]
    Instance(#[from] InstanceCreationError),

    #[error("debug callback creation failure: {0}")]
    DebugCallback(#[from] DebugCallbackCreationError),

    #[error("surface creation failure: {0}")]
    Surface(#[from] vulkano_win::CreationError),

    #[error("no suitable physical device were found")]
    NoSuitablePhysicalDevice,

    #[error("device creation failure: {0}")]
    Device(#[from] DeviceCreationError),

    #[error("failed to get surface capabilities: {0}")]
    SurfaceCapabilities(#[from] CapabilitiesError),

    #[error("swapchain creation failure: {0}")]
    Swapchain(#[from] SwapchainCreationError),

    #[error("depth image creation failure: {0}")]
    DepthImage(#[from] ImageCreationError),

    #[error("render pass creation failure: {0}")]
    RenderPass(#[from] RenderPassCreationError),

    #[error("framebuffer creation failure: {0}")]
    Framebuffer(#[from] FramebuffersCreationError),

    #[error("graphics pipeline creation failure: {0}")]
    GraphicsPipeline(#[from] GraphicsPipelineCreationError),

    #[error("sampler creation failure: {0}")]
    Sampler(#[from] SamplerCreationError),

    #[error("descriptor set creation failure: {0}")]
    DescriptorSet(#[from] DescriptorSetCreationError),

    #[error("failed to allocate device memory: {0}")]
    MemoryAllocation(#[from] DeviceMemoryAllocError),

    #[error("failed to execute GpuFuture: {0}")]
    GpuFutureFlush(#[from] FlushError),
}

#[derive(Debug, Error)]
pub enum DescriptorSetCreationError {
    #[error("persistent descriptor set addition failure: {0}")]
    Addition(#[from] PersistentDescriptorSetError),

    #[error("persistent descriptor set build failure: {0}")]
    Build(#[from] PersistentDescriptorSetBuildError),
}

#[derive(Debug, Error)]
pub enum FramebuffersCreationError {
    #[error("image view creation failure: {0}")]
    ImageViewCreation(#[from] ImageViewCreationError),

    #[error("framebuffer creation failure: {0}")]
    FramebufferSet(#[from] FramebufferCreationError),
}

#[derive(Debug, Error)]
pub enum ResizeError {
    #[error("swapchain recreation failure: {0}")]
    SwapchainRecreation(#[from] SwapchainCreationError),

    #[error("framebuffers recreation failure: {0}")]
    FramebuffersRecreation(#[from] FramebuffersCreationError),

    #[error("depth image recreation failure: {0}")]
    DepthImageRecreation(#[from] ImageCreationError),
}

#[derive(Debug, Error)]
pub enum TransferCommandBufferCreationError {
    #[error("failed to allocate transfer command buffer: {0}")]
    OomError(#[from] OomError),

    #[error("update buffer command failure: {0}")]
    UpdateBuffer(#[from] UpdateBufferError),

    #[error("transfer command buffer build failure: {0}")]
    Build(#[from] BuildError),
}

#[derive(Debug, Error)]
pub enum GraphicsCommandBufferCreationError {
    #[error("failed to allocate graphics command buffer: {0}")]
    OomError(#[from] OomError),

    #[error("begin render pass command failure: {0}")]
    BeginRenderPass(#[from] BeginRenderPassError),

    #[error("draw indexed command failure: {0}")]
    DrawIndexed(#[from] DrawIndexedError),

    #[error("failed to allocate device memory: {0}")]
    MemoryAllocation(#[from] DeviceMemoryAllocError),

    #[error("graphics command buffer image creation failure: {0}")]
    ImageCreation(#[from] ImageCreationError),

    #[error("graphics command buffer image view creation failure: {0}")]
    ImageViewCreation(#[from] ImageViewCreationError),

    #[error("failed to execute GpuFuture: {0}")]
    GpuFutureFlush(#[from] FlushError),

    #[error("graphics command buffer descriptor set creation failure: {0}")]
    DescriptorSet(#[from] DescriptorSetCreationError),

    #[error("graphics command buffer building failure: {0}")]
    Building(#[from] AutoCommandBufferBuilderContextError),

    #[error("graphics command buffer build failure: {0}")]
    Build(#[from] BuildError),
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("transfer command buffer creation error while rendering: {0}")]
    TransferCommandBufferCreation(#[from] TransferCommandBufferCreationError),

    #[error("graphics command buffer creation error while rendering: {0}")]
    GraphicsCommandBufferCreation(#[from] GraphicsCommandBufferCreationError),

    #[error("acquiring next image failure while rendering: {0}")]
    AcquireNextImage(#[from] AcquireError),

    #[error("command buffer execution failure while rendering: {0}")]
    CommandBufferExecution(#[from] CommandBufferExecError),

    #[error("failed to submit commands while rendering: {0}")]
    SubmitQueue(#[from] FlushError),

    #[error("failed to resize while rendering: {0}")]
    Resize(#[from] ResizeError),
}
