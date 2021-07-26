use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::super::{
    command::{self, CommandBuffer},
    device::{self, Device},
    slotmap::SlotMappable,
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
        device_key: device::Key,
        create_info: &vk::CommandPoolCreateInfo,
    ) -> Result<Key> {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");
        let handle = device.loader().create_command_pool(create_info, None)?;

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            parent_device: device_key,
        });
        Ok(key)
    }

    pub fn handle(&self) -> vk::CommandPool {
        self.handle
    }

    pub fn parent_device(&self) -> device::Key {
        self.parent_device
    }

    pub fn allocate_command_buffers(&self, count: u32) -> Result<Vec<command::buffer::Key>> {
        let device_key = self.parent_device();
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.handle())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);
        let loader = device.loader();
        unsafe {
            loader
                .allocate_command_buffers(&allocate_info)?
                .into_iter()
                .map(|command_buffer| CommandBuffer::new(self.key, command_buffer))
                .collect()
        }
    }

    pub unsafe fn free_command_buffers(&self, command_buffers: &[command::buffer::Key]) {
        let device_key = self.parent_device();
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");

        let mut slotmap_command_buffer = SlotMappable::slotmap().write().unwrap();
        let command_buffers: Vec<_> = command_buffers
            .iter()
            .map(|key| {
                let command_buffer: &CommandBuffer = slotmap_command_buffer
                    .get(*key)
                    .expect("command buffer not found");
                let handle = command_buffer.handle();
                slotmap_command_buffer.remove(*key);
                handle
            })
            .collect();

        device
            .loader()
            .free_command_buffers(self.handle(), command_buffers.as_slice());
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(self.parent_device())
            .expect("device not found");
        let loader = device.loader();
        unsafe { loader.destroy_command_pool(self.handle, None) }
    }
}
