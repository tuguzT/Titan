use std::ops::Deref;
use std::sync::Mutex;

use ash::vk;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::super::{
    command::{self, CommandBuffers},
    device::{self, Device},
    slotmap::{HasParent, SlotMappable},
    utils::{HasHandle, HasLoader},
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct CommandPool {
    #[key]
    key: Key,
    handle: Mutex<vk::CommandPool>,
    parent_device: device::Key,
}

impl HasParent<Device> for CommandPool {
    fn parent_key(&self) -> device::Key {
        self.parent_device
    }
}

impl HasHandle for CommandPool {
    type Handle = vk::CommandPool;

    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_> {
        Box::new(self.handle.lock().unwrap())
    }
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
            handle: Mutex::new(handle),
            parent_device: device_key,
        });
        Ok(key)
    }

    pub fn allocate_command_buffers(&self, count: u32) -> Result<command::buffers::Key> {
        let device_key = self.parent_key();
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");

        let handle = self.handle();
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(**handle)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);
        let loader = device.loader();
        unsafe {
            let handles = loader.allocate_command_buffers(&allocate_info)?;
            CommandBuffers::new(&handles, self.key)
        }
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(self.parent_key())
            .expect("device not found");
        let loader = device.loader();
        let handle = self.handle();
        unsafe { loader.destroy_command_pool(**handle, None) }
    }
}
