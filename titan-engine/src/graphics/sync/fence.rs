use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::super::{
    device::{self, Device},
    slotmap::SlotMappable,
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct Fence {
    key: Key,
    handle: vk::Fence,
    parent_device: device::Key,
}

impl Fence {
    pub fn new(device_key: device::Key, create_info: &vk::FenceCreateInfo) -> Result<Key> {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");
        let handle = unsafe { device.loader().create_fence(&create_info, None)? };

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            parent_device: device_key,
        });
        Ok(key)
    }

    pub fn handle(&self) -> vk::Fence {
        self.handle
    }

    pub fn parent_device(&self) -> device::Key {
        self.parent_device
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(self.parent_device())
            .expect("device not found");
        unsafe { device.loader().destroy_fence(self.handle, None) }
    }
}
