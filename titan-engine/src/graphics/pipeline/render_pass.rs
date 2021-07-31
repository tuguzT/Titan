use std::ops::Deref;

use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::super::{
    command::CommandBuffer,
    device::Device,
    ext::swapchain::{self, Swapchain},
    slotmap::{HasParent, SlotMappable},
    utils::{HasHandle, HasLoader},
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct RenderPass {
    #[key]
    key: Key,
    handle: vk::RenderPass,
    parent_swapchain: swapchain::Key,
}

impl HasParent<Swapchain> for RenderPass {
    fn parent_key(&self) -> swapchain::Key {
        self.parent_swapchain
    }
}

impl HasHandle for RenderPass {
    type Handle = vk::RenderPass;

    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_> {
        Box::new(&self.handle)
    }
}

impl RenderPass {
    pub fn new(swapchain_key: swapchain::Key) -> Result<Key> {
        let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
        let swapchain: &Swapchain = slotmap_swapchain
            .get(swapchain_key)
            .expect("swapchain not found");

        let device_key = <Swapchain as HasParent<Device>>::parent_key(swapchain);
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

    pub unsafe fn begin(
        &self,
        command_buffer: &CommandBuffer,
        begin_info: &vk::RenderPassBeginInfo,
        contents: vk::SubpassContents,
    ) -> Result<()> {
        let swapchain_key = self.parent_key();
        let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
        let swapchain: &Swapchain = slotmap_swapchain
            .get(swapchain_key)
            .expect("swapchain not found");

        let device_key = <Swapchain as HasParent<Device>>::parent_key(swapchain);
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");
        let loader = device.loader();
        let handle = command_buffer.handle();

        loader.cmd_begin_render_pass(**handle, begin_info, contents);
        Ok(())
    }

    pub unsafe fn end(&self, command_buffer: &CommandBuffer) -> Result<()> {
        let swapchain_key = self.parent_key();
        let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
        let swapchain: &Swapchain = slotmap_swapchain
            .get(swapchain_key)
            .expect("swapchain not found");

        let device_key = <Swapchain as HasParent<Device>>::parent_key(swapchain);
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");
        let loader = device.loader();
        let handle = command_buffer.handle();

        loader.cmd_end_render_pass(**handle);
        Ok(())
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        let swapchain_key = self.parent_key();
        let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
        let swapchain: &Swapchain = slotmap_swapchain
            .get(swapchain_key)
            .expect("swapchain not found");

        let device_key = <Swapchain as HasParent<Device>>::parent_key(swapchain);
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");
        let loader = device.loader();

        unsafe { loader.destroy_render_pass(self.handle, None) }
    }
}
