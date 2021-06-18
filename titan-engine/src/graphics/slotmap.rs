use std::sync::RwLock;

use slotmap::SlotMap;

use super::{
    command::{CommandBuffer, CommandPool},
    device::{Device, PhysicalDevice, Queue},
    ext::{DebugUtils, Swapchain},
    framebuffer::Framebuffer,
    image::{Image, ImageView},
    instance::Instance,
    pipeline::{GraphicsPipeline, PipelineLayout, RenderPass},
    shader::ShaderModule,
    surface::Surface,
    sync::{Fence, Semaphore},
};

pub fn clear() {
    clear_slot_mappable::<Fence>();
    clear_slot_mappable::<Semaphore>();
    clear_slot_mappable::<ShaderModule>();
    clear_slot_mappable::<CommandBuffer>();
    clear_slot_mappable::<CommandPool>();
    clear_slot_mappable::<Framebuffer>();
    clear_slot_mappable::<GraphicsPipeline>();
    clear_slot_mappable::<PipelineLayout>();
    clear_slot_mappable::<RenderPass>();
    clear_slot_mappable::<ImageView>();
    clear_slot_mappable::<Image>();
    clear_slot_mappable::<Swapchain>();
    clear_slot_mappable::<Queue>();
    clear_slot_mappable::<Device>();
    clear_slot_mappable::<PhysicalDevice>();
    clear_slot_mappable::<Surface>();
    clear_slot_mappable::<DebugUtils>();
    clear_slot_mappable::<Instance>();
}

fn clear_slot_mappable<T>()
where
    T: SlotMappable + 'static,
{
    let mut slotmap = T::slotmap().write().unwrap();
    slotmap.clear()
}

pub trait SlotMappable: Sized + Send + Sync {
    type Key: slotmap::Key;

    fn key(&self) -> Self::Key;

    fn slotmap() -> &'static RwLock<SlotMap<Self::Key, Self>>;
}
