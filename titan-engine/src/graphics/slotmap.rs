use std::sync::RwLock;

use slotmap::SlotMap;

use super::ext::Swapchain;
use super::{DebugUtils, Device, Framebuffer, Instance, PhysicalDevice, Queue, Surface};
use crate::graphics::commands::{CommandBuffer, CommandPool};
use crate::graphics::image::{Image, ImageView};
use crate::graphics::pipeline::{GraphicsPipeline, PipelineLayout, RenderPass};
use crate::graphics::sync::{Fence, Semaphore};

slotmap::new_key_type! {
    pub struct InstanceKey;
    pub struct DebugUtilsKey;
    pub struct SurfaceKey;
    pub struct PhysicalDeviceKey;
    pub struct DeviceKey;
    pub struct QueueKey;
    pub struct SwapchainKey;
    pub struct ImageKey;
    pub struct ImageViewKey;
    pub struct RenderPassKey;
    pub struct PipelineLayoutKey;
    pub struct GraphicsPipelineKey;
    pub struct FramebufferKey;
    pub struct CommandPoolKey;
    pub struct CommandBufferKey;
    pub struct SemaphoreKey;
    pub struct FenceKey;
}

lazy_static::lazy_static! {
    pub static ref SLOTMAP_INSTANCE: RwLock<SlotMap<InstanceKey, Instance>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_DEBUG_UTILS: RwLock<SlotMap<DebugUtilsKey, DebugUtils>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_SURFACE: RwLock<SlotMap<SurfaceKey, Surface>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_PHYSICAL_DEVICE: RwLock<SlotMap<PhysicalDeviceKey, PhysicalDevice>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_DEVICE: RwLock<SlotMap<DeviceKey, Device>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_QUEUE: RwLock<SlotMap<QueueKey, Queue>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_SWAPCHAIN: RwLock<SlotMap<SwapchainKey, Swapchain>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_IMAGE: RwLock<SlotMap<ImageKey, Image>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_IMAGE_VIEW: RwLock<SlotMap<ImageViewKey, ImageView>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_RENDER_PASS: RwLock<SlotMap<RenderPassKey, RenderPass>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_PIPELINE_LAYOUT: RwLock<SlotMap<PipelineLayoutKey, PipelineLayout>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_GRAPHICS_PIPELINE: RwLock<SlotMap<GraphicsPipelineKey, GraphicsPipeline>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_FRAMEBUFFER: RwLock<SlotMap<FramebufferKey, Framebuffer>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_COMMAND_POOL: RwLock<SlotMap<CommandPoolKey, CommandPool>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_COMMAND_BUFFER: RwLock<SlotMap<CommandBufferKey, CommandBuffer>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_SEMAPHORE: RwLock<SlotMap<SemaphoreKey, Semaphore>> = RwLock::new(SlotMap::with_key());
    pub static ref SLOTMAP_FENCE: RwLock<SlotMap<FenceKey, Fence>> = RwLock::new(SlotMap::with_key());
}

pub unsafe fn destroy() {
    if let Ok(mut slotmap) = SLOTMAP_FENCE.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_SEMAPHORE.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_COMMAND_BUFFER.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_COMMAND_POOL.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_FRAMEBUFFER.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_GRAPHICS_PIPELINE.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_PIPELINE_LAYOUT.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_RENDER_PASS.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_IMAGE_VIEW.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_IMAGE.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_SWAPCHAIN.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_QUEUE.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_DEVICE.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_PHYSICAL_DEVICE.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_SURFACE.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_DEBUG_UTILS.write() {
        slotmap.clear()
    }
    if let Ok(mut slotmap) = SLOTMAP_INSTANCE.write() {
        slotmap.clear()
    }
}
