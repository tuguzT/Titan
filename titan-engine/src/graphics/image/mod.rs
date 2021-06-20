use std::error::Error;

use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;
pub use view::ImageView;

use super::{
    device::{self, Device},
    slotmap::SlotMappable,
};

slotmap::new_key_type! {
    pub struct Key;
}

pub mod view;

#[derive(SlotMappable)]
pub struct Image {
    key: Key,
    handle: vk::Image,
    parent_device: device::Key,
    owned: bool,
}

impl Image {
    pub unsafe fn new(
        device_key: device::Key,
        create_info: &vk::ImageCreateInfo,
    ) -> Result<Key, Box<dyn Error>> {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");
        let handle = device.loader().create_image(create_info, None)?;

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            parent_device: device_key,
            owned: false,
        });
        Ok(key)
    }

    pub unsafe fn from_raw(
        device_key: device::Key,
        handle: vk::Image,
    ) -> Result<Key, Box<dyn Error>> {
        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            parent_device: device_key,
            owned: true,
        });
        Ok(key)
    }

    pub fn handle(&self) -> vk::Image {
        self.handle
    }

    pub fn parent_device(&self) -> device::Key {
        self.parent_device
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(self.parent_device())
            .expect("device not found");
        if !self.owned {
            unsafe { device.loader().destroy_image(self.handle, None) }
        }
    }
}
