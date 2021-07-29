use std::ops::Deref;
use std::sync::Mutex;

use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::super::{
    command::{self, CommandPool},
    device::Device,
    slotmap::{HasParent, SlotMappable},
    utils::{HasHandle, HasLoader},
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

impl HasParent<CommandPool> for CommandBuffer {
    fn parent_key(&self) -> command::pool::Key {
        self.parent_command_pool
    }
}

impl HasHandle for CommandBuffer {
    type Handle = vk::CommandBuffer;

    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_> {
        Box::new(self.handle.lock().unwrap())
    }
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

    pub unsafe fn begin(&self, begin_info: &vk::CommandBufferBeginInfo) -> Result<()> {
        let slotmap_command_pool = SlotMappable::slotmap().read().unwrap();
        let command_pool: &CommandPool = slotmap_command_pool
            .get(self.parent_key())
            .expect("parent was lost");

        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(command_pool.parent_key())
            .expect("command pool parent was lost");

        let loader = device.loader();
        let handle = self.handle();
        Ok(loader.begin_command_buffer(**handle, begin_info)?)
    }

    pub unsafe fn end(&self) -> Result<()> {
        let slotmap_command_pool = SlotMappable::slotmap().read().unwrap();
        let command_pool: &CommandPool = slotmap_command_pool
            .get(self.parent_key())
            .expect("parent was lost");

        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(command_pool.parent_key())
            .expect("command pool parent was lost");

        let loader = device.loader();
        let handle = self.handle();
        Ok(loader.end_command_buffer(**handle)?)
    }
}
