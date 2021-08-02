use std::ops::Deref;

use ash::vk;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::{
    device::{self, Device},
    slotmap::{HasParent, SlotMappable},
    utils::{HasHandle, HasLoader},
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct Framebuffer {
    #[key]
    key: Key,
    handle: vk::Framebuffer,
    parent_device: device::Key,
}

impl HasParent<Device> for Framebuffer {
    fn parent_key(&self) -> device::Key {
        self.parent_device
    }
}

impl HasHandle for Framebuffer {
    type Handle = vk::Framebuffer;

    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_> {
        Box::new(&self.handle)
    }
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
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(self.parent_key())
            .expect("device not found");
        let loader = device.loader();
        unsafe { loader.destroy_framebuffer(self.handle, None) }
    }
}
