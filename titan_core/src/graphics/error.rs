//! Error types and utilities for graphics backend for game engine.

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

/// Error that can happen when creating the [`Renderer`](super::Renderer) system.
#[derive(Debug, Error)]
pub enum RendererCreationError {
    #[error("failed to allocate memory for shader modules: {0}")]
    OutOfMemory(#[from] OomError),

    #[error("instance creation failure: {0}")]
    InstanceCreation(#[from] InstanceCreationError),

    #[error("debug callback creation failure: {0}")]
    DebugCallbackCreation(#[from] DebugCallbackCreationError),

    #[error("surface creation failure: {0}")]
    SurfaceCreation(#[from] vulkano_win::CreationError),

    #[error("no suitable physical device were found")]
    NoSuitablePhysicalDevice,

    #[error("device creation failure: {0}")]
    DeviceCreation(#[from] DeviceCreationError),

    #[error("failed to get surface capabilities: {0}")]
    SurfaceCapabilitiesRetrieve(#[from] CapabilitiesError),

    #[error("swapchain creation failure: {0}")]
    SwapchainCreation(#[from] SwapchainCreationError),

    #[error("image creation failure: {0}")]
    ImageCreation(#[from] ImageCreationError),

    #[error("render pass creation failure: {0}")]
    RenderPassCreation(#[from] RenderPassCreationError),

    #[error("framebuffer creation failure: {0}")]
    FramebuffersCreation(#[from] FramebuffersCreationError),

    #[error("graphics pipeline creation failure: {0}")]
    GraphicsPipelineCreation(#[from] GraphicsPipelineCreationError),

    #[error("sampler creation failure: {0}")]
    SamplerCreation(#[from] SamplerCreationError),

    #[error("descriptor set creation failure: {0}")]
    DescriptorSetCreation(#[from] DescriptorSetCreationError),

    #[error("failed to allocate device memory: {0}")]
    MemoryAllocation(#[from] DeviceMemoryAllocError),

    #[error("failed to execute GpuFuture: {0}")]
    GpuFutureFlush(#[from] FlushError),
}

/// Error that can happen on descriptor set creation.
#[derive(Debug, Error)]
pub enum DescriptorSetCreationError {
    #[error("persistent descriptor set addition failure: {0}")]
    Addition(#[from] PersistentDescriptorSetError),

    #[error("persistent descriptor set build failure: {0}")]
    Build(#[from] PersistentDescriptorSetBuildError),
}

/// Error that can happen on framebuffers creation for [`Renderer`](super::Renderer) system.
#[derive(Debug, Error)]
pub enum FramebuffersCreationError {
    #[error("image view creation failure: {0}")]
    ImageViewCreation(#[from] ImageViewCreationError),

    #[error("framebuffer creation failure: {0}")]
    FramebufferSetup(#[from] FramebufferCreationError),
}

/// Error that can happen on resizing of [`Renderer`](super::Renderer) system.
#[derive(Debug, Error)]
pub enum ResizeError {
    #[error("swapchain recreation failure: {0}")]
    SwapchainRecreation(#[from] SwapchainCreationError),

    #[error("framebuffers recreation failure: {0}")]
    FramebuffersRecreation(#[from] FramebuffersCreationError),

    #[error("depth image recreation failure: {0}")]
    DepthImageRecreation(#[from] ImageCreationError),
}

/// Error that can happen on transfer command buffer creation
/// for [`Renderer`](super::Renderer) system.
///
#[derive(Debug, Error)]
pub enum TransferCommandBufferCreationError {
    #[error("failed to allocate transfer command buffer: {0}")]
    OutOfMemory(#[from] OomError),

    #[error("update buffer command failure: {0}")]
    UpdateBuffer(#[from] UpdateBufferError),

    #[error("transfer command buffer build failure: {0}")]
    Build(#[from] BuildError),
}

/// Error that can happen on graphics command buffer creation
/// for [`Renderer`](super::Renderer) system.
///
#[derive(Debug, Error)]
pub enum GraphicsCommandBufferCreationError {
    #[error("failed to allocate graphics command buffer: {0}")]
    OutOfMemory(#[from] OomError),

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
    DescriptorSetCreation(#[from] DescriptorSetCreationError),

    #[error("graphics command buffer building failure: {0}")]
    WrongUsage(#[from] AutoCommandBufferBuilderContextError),

    #[error("graphics command buffer build failure: {0}")]
    Build(#[from] BuildError),
}

/// Error that can happen on rendering operation of [`Renderer`](super::Renderer) system.
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
