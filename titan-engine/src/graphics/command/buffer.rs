use std::sync::{Mutex, MutexGuard};

use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::super::{
    command::{self, CommandPool},
    device::Device,
    slotmap::SlotMappable,
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct CommandBuffer {
    key: Key,
    handle: Mutex<vk::CommandBuffer>,
    parent_command_pool: command::pool::Key,
}

impl CommandBuffer {
    pub(super) unsafe fn new(
        command_pool_key: command::pool::Key,
        handle: vk::CommandBuffer,
    ) -> Result<Key> {
        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle: Mutex::new(handle),
            parent_command_pool: command_pool_key,
        });
        Ok(key)
    }

    pub fn parent_command_pool(&self) -> command::pool::Key {
        self.parent_command_pool
    }

    pub fn handle(&self) -> MutexGuard<vk::CommandBuffer> {
        self.handle.lock().unwrap()
    }

    pub unsafe fn begin(&self, begin_info: &vk::CommandBufferBeginInfo) -> Result<()> {
        let slotmap_command_pool = SlotMappable::slotmap().read().unwrap();
        let command_pool: &CommandPool = slotmap_command_pool
            .get(self.parent_command_pool())
            .expect("parent was lost");

        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(command_pool.parent_device())
            .expect("command pool parent was lost");

        let loader = device.loader();
        Ok(loader.begin_command_buffer(*self.handle(), begin_info)?)
    }

    pub unsafe fn end(&self) -> Result<()> {
        let slotmap_command_pool = SlotMappable::slotmap().read().unwrap();
        let command_pool: &CommandPool = slotmap_command_pool
            .get(self.parent_command_pool())
            .expect("parent was lost");

        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(command_pool.parent_device())
            .expect("command pool parent was lost");

        let loader = device.loader();
        Ok(loader.end_command_buffer(*self.handle())?)
    }
}
