use std::ops::Deref;

use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::super::{
    device::Device,
    image::{self, Image},
    slotmap::{HasParent, SlotMappable},
    utils::{HasHandle, HasLoader},
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct ImageView {
    #[key]
    key: Key,
    handle: vk::ImageView,
    parent_image: image::Key,
}

impl HasParent<Image> for ImageView {
    fn parent_key(&self) -> image::Key {
        self.parent_image
    }
}

impl HasHandle for ImageView {
    type Handle = vk::ImageView;

    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_> {
        Box::new(&self.handle)
    }
}

impl ImageView {
    pub unsafe fn new(image_key: image::Key, create_info: &vk::ImageViewCreateInfo) -> Result<Key> {
        let slotmap_image = SlotMappable::slotmap().read().unwrap();
        let image: &Image = slotmap_image.get(image_key).expect("image not found");

        let device_key = image.parent_key();
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");

        let handle = device.loader().create_image_view(create_info, None)?;

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            parent_image: image_key,
        });
        Ok(key)
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        let slotmap_image = SlotMappable::slotmap().read().unwrap();
        let image: &Image = slotmap_image
            .get(self.parent_key())
            .expect("image not found");

        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(image.parent_key())
            .expect("device not found");
        let loader = device.loader();

        unsafe { loader.destroy_image_view(self.handle, None) }
    }
}
