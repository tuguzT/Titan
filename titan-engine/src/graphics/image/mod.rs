use std::ops::Deref;

use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;
pub use view::ImageView;

use crate::error::Result;

use super::{
    device::{self, Device},
    slotmap::{HasParent, SlotMappable},
    utils::{HasHandle, HasLoader},
};

slotmap::new_key_type! {
    pub struct Key;
}

pub mod view;

#[derive(SlotMappable)]
pub struct Image {
    #[key]
    key: Key,
    handle: vk::Image,
    parent_device: device::Key,
    owned: bool,
}

impl HasParent<Device> for Image {
    fn parent_key(&self) -> device::Key {
        self.parent_device
    }
}

impl HasHandle for Image {
    type Handle = vk::Image;

    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_> {
        Box::new(&self.handle)
    }
}

impl Image {
    pub unsafe fn new(device_key: device::Key, create_info: &vk::ImageCreateInfo) -> Result<Key> {
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

    pub unsafe fn from_raw(device_key: device::Key, handle: vk::Image) -> Result<Key> {
        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            parent_device: device_key,
            owned: true,
        });
        Ok(key)
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(self.parent_key())
            .expect("device not found");
        if !self.owned {
            unsafe { device.loader().destroy_image(self.handle, None) }
        }
    }
}
