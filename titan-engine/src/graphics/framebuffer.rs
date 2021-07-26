use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::{
    device::{self, Device},
    slotmap::SlotMappable,
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct Framebuffer {
    key: Key,
    handle: vk::Framebuffer,
    parent_device: device::Key,
}

impl Framebuffer {
    pub unsafe fn new(
        device_key: device::Key,
        create_info: &vk::FramebufferCreateInfo,
    ) -> Result<Key> {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");
        let handle = device.loader().create_framebuffer(create_info, None)?;

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            parent_device: device_key,
        });
        Ok(key)
    }

    pub fn handle(&self) -> vk::Framebuffer {
        self.handle
    }

    pub fn parent_device(&self) -> device::Key {
        self.parent_device
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(self.parent_device())
            .expect("device not found");
        let loader = device.loader();
        unsafe { loader.destroy_framebuffer(self.handle, None) }
    }
}
