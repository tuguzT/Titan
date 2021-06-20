use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::super::{ext::Swapchain, swapchain, CommandBuffer, Device, SlotMappable};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct RenderPass {
    key: Key,
    handle: vk::RenderPass,
    parent_swapchain: swapchain::Key,
}

impl RenderPass {
    pub fn new(swapchain_key: swapchain::Key) -> Result<Key> {
        let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
        let swapchain: &Swapchain = slotmap_swapchain
            .get(swapchain_key)
            .expect("swapchain not found");

        let device_key = swapchain.parent_device();
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");

        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain.format().format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
        let color_attachments = [*color_attachment];

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let color_attachment_refs = [*color_attachment_ref];

        let subpass_description = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs);
        let subpasses = [*subpass_description];

        let subpass_dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::default())
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
        let dependencies = [*subpass_dependency];

        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&color_attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);
        let handle = unsafe { device.loader().create_render_pass(&create_info, None)? };

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            parent_swapchain: swapchain_key,
        });
        Ok(key)
    }

    pub fn handle(&self) -> vk::RenderPass {
        self.handle
    }

    pub fn parent_swapchain(&self) -> swapchain::Key {
        self.parent_swapchain
    }

    pub unsafe fn begin(
        &self,
        command_buffer: &CommandBuffer,
        begin_info: &vk::RenderPassBeginInfo,
        contents: vk::SubpassContents,
    ) -> Result<()> {
        let swapchain_key = self.parent_swapchain();
        let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
        let swapchain: &Swapchain = slotmap_swapchain
            .get(swapchain_key)
            .expect("swapchain not found");

        let device_key = swapchain.parent_device();
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");

        Ok(device
            .loader()
            .cmd_begin_render_pass(command_buffer.handle(), &begin_info, contents))
    }

    pub unsafe fn end(&self, command_buffer: &CommandBuffer) -> Result<()> {
        let swapchain_key = self.parent_swapchain();
        let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
        let swapchain: &Swapchain = slotmap_swapchain
            .get(swapchain_key)
            .expect("swapchain not found");

        let device_key = swapchain.parent_device();
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");

        Ok(device.loader().cmd_end_render_pass(command_buffer.handle()))
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        let swapchain_key = self.parent_swapchain();
        let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
        let swapchain: &Swapchain = slotmap_swapchain
            .get(swapchain_key)
            .expect("swapchain not found");

        let device_key = swapchain.parent_device();
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");

        unsafe { device.loader().destroy_render_pass(self.handle, None) }
    }
}
