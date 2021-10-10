//! Error types and utilities for graphics backend for game engine.

use thiserror::Error;
use vulkano::command_buffer::{BuildError, CommandBufferExecError, UpdateBufferError};
use vulkano::descriptor_set::DescriptorSetError;
use vulkano::device::DeviceCreationError;
use vulkano::instance::debug::DebugCallbackCreationError;
use vulkano::instance::InstanceCreationError;
use vulkano::memory::DeviceMemoryAllocError;
use vulkano::swapchain::{AcquireError, CapabilitiesError, SwapchainCreationError};
use vulkano::sync::FlushError;
use vulkano::OomError;

use crate::graphics::frame::{
    object_draw::error::{ObjectDrawError, ObjectDrawSystemCreationError},
    system::error::{
        DrawPassExecuteError, FrameCreationError, FrameSystemCreationError, NextPassError,
    },
    ui_draw::error::{UiDrawError, UiDrawSystemCreationError},
};

/// Error that can happen when creating the [`Renderer`](super::Renderer) system.
#[derive(Debug, Error)]
pub enum RendererCreationError {
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

    #[error("failed to allocate device memory: {0}")]
    MemoryAllocation(#[from] DeviceMemoryAllocError),

    #[error("frame system creation failure: {0}")]
    FrameSystemCreation(#[from] FrameSystemCreationError),

    #[error("object draw system creation failure: {0}")]
    ObjectDrawSystemCreation(#[from] ObjectDrawSystemCreationError),

    #[error("UI draw system creation failure: {0}")]
    UiDrawSystemCreation(#[from] UiDrawSystemCreationError),
}

/// Error that can happen on descriptor set creation.
#[derive(Debug, Error)]
pub enum DescriptorSetCreationError {
    #[error("persistent descriptor set build failure: {0}")]
    Build(#[from] DescriptorSetError),
}

/// Error that can happen on resizing of [`Renderer`](super::Renderer) system.
#[derive(Debug, Error)]
pub enum ResizeError {
    #[error("swapchain recreation failure: {0}")]
    SwapchainRecreation(#[from] SwapchainCreationError),
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

/// Error that can happen on rendering operation of [`Renderer`](super::Renderer) system.
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("transfer command buffer creation error while rendering: {0}")]
    TransferCommandBufferCreation(#[from] TransferCommandBufferCreationError),

    #[error("acquiring next image failure while rendering: {0}")]
    AcquireNextImage(#[from] AcquireError),

    #[error("command buffer execution failure while rendering: {0}")]
    CommandBufferExecution(#[from] CommandBufferExecError),

    #[error("failed to submit commands while rendering: {0}")]
    SubmitQueue(#[from] FlushError),

    #[error("frame creation failure: {0}")]
    FrameCreation(#[from] FrameCreationError),

    #[error("subpass switching failure: {0}")]
    NextPass(#[from] NextPassError),

    #[error("failed to draw game objects: {0}")]
    ObjectDraw(#[from] ObjectDrawError),

    #[error("failed to draw UI: {0}")]
    UiDraw(#[from] UiDrawError),

    #[error("failed to execute draw command buffer: {0}")]
    DrawPassExecution(#[from] DrawPassExecuteError),

    #[error("failed to resize while rendering: {0}")]
    Resize(#[from] ResizeError),
}
