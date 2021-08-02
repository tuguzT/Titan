use std::ops::Deref;

use ash::vk;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::super::{
    device::{self, Device},
    slotmap::{HasParent, SlotMappable},
    utils::{HasHandle, HasLoader},
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct Fence {
    #[key]
    key: Key,
    handle: vk::Fence,
    parent_device: device::Key,
}

impl HasParent<Device> for Fence {
    fn parent_key(&self) -> device::Key {
        self.parent_device
    }
}

impl HasHandle for Fence {
    type Handle = vk::Fence;

    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_> {
        Box::new(&self.handle)
    }
}

impl Fence {
    pub fn new(device_key: device::Key, create_info: &vk::FenceCreateInfo) -> Result<Key> {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");
        let handle = unsafe { device.loader().create_fence(create_info, None)? };

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            parent_device: device_key,
        });
        Ok(key)
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(self.parent_key())
            .expect("device not found");
        let loader = device.loader();
        unsafe { loader.destroy_fence(self.handle, None) }
    }
}
