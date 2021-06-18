use std::error::Error;

use ::slotmap::Key as SlotMapKey;
use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use super::super::{
    super::slotmap::SlotMappable, command, command::CommandBuffer, device, device::Device, utils,
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct CommandPool {
    key: Key,
    handle: vk::CommandPool,
    parent_device: device::Key,
}

impl CommandPool {
    pub unsafe fn new(
        key: Key,
        device_key: device::Key,
        create_info: &vk::CommandPoolCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let slotmap_device = Device::slotmap().read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

        let handle = device.loader().create_command_pool(create_info, None)?;
        Ok(Self {
            key,
            handle,
            parent_device: device_key,
        })
    }

    pub fn handle(&self) -> vk::CommandPool {
        self.handle
    }

    pub fn parent_device(&self) -> device::Key {
        self.parent_device
    }

    pub fn enumerate_command_buffers(
        &self,
        count: u32,
    ) -> Result<Vec<CommandBuffer>, Box<dyn Error>> {
        let device_key = self.parent_device();
        let slotmap_device = Device::slotmap().read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("parent was lost"))?;

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.handle())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);
        Ok(unsafe {
            device
                .loader()
                .allocate_command_buffers(&allocate_info)?
                .into_iter()
                .map(|command_buffer| {
                    CommandBuffer::new(command::buffer::Key::null(), self.key, command_buffer)
                })
                .collect()
        })
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        let slotmap_device = Device::slotmap().read();
        let slotmap_device = match slotmap_device {
            Ok(value) => value,
            Err(_) => return,
        };
        let device = match slotmap_device.get(self.parent_device()) {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_command_pool(self.handle, None) }
    }
}
