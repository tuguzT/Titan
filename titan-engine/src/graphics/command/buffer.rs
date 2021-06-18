use std::error::Error;

use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use super::super::{
    super::slotmap::SlotMappable, command, command::CommandPool, device::Device, utils,
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct CommandBuffer {
    key: Key,
    handle: vk::CommandBuffer,
    parent_command_pool: command::pool::Key,
}

impl CommandBuffer {
    pub(super) unsafe fn new(
        key: Key,
        command_pool_key: command::pool::Key,
        handle: vk::CommandBuffer,
    ) -> Self {
        Self {
            key,
            handle,
            parent_command_pool: command_pool_key,
        }
    }

    pub fn parent_command_pool(&self) -> command::pool::Key {
        self.parent_command_pool
    }

    pub fn handle(&self) -> vk::CommandBuffer {
        self.handle
    }

    pub unsafe fn begin(
        &self,
        begin_info: &vk::CommandBufferBeginInfo,
    ) -> Result<(), Box<dyn Error>> {
        let slotmap_command_pool = CommandPool::slotmap().read()?;
        let command_pool = slotmap_command_pool
            .get(self.parent_command_pool())
            .ok_or_else(|| utils::make_error("parent was lost"))?;

        let slotmap_device = Device::slotmap().read()?;
        let device = slotmap_device
            .get(command_pool.parent_device())
            .ok_or_else(|| utils::make_error("command pool parent was lost"))?;

        Ok(device
            .loader()
            .begin_command_buffer(self.handle, begin_info)?)
    }

    pub unsafe fn end(&self) -> Result<(), Box<dyn Error>> {
        let slotmap_command_pool = CommandPool::slotmap().read()?;
        let command_pool = slotmap_command_pool
            .get(self.parent_command_pool())
            .ok_or_else(|| utils::make_error("parent was lost"))?;

        let slotmap_device = Device::slotmap().read()?;
        let device = slotmap_device
            .get(command_pool.parent_device())
            .ok_or_else(|| utils::make_error("command pool parent was lost"))?;

        Ok(device.loader().end_command_buffer(self.handle)?)
    }
}
