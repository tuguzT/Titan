use std::error::Error;

use ash::version::DeviceV1_0;
use ash::vk;

use super::{device, image, utils};

pub mod slotmap;
pub mod view;

pub struct Image {
    handle: vk::Image,
    parent_device: device::logical::slotmap::Key,
    owned: bool,
}

impl Image {
    pub unsafe fn new(
        device_key: device::logical::slotmap::Key,
        create_info: &vk::ImageCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let slotmap_device = device::logical::slotmap::read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;
        let handle = device.loader().create_image(create_info, None)?;
        Ok(Self {
            handle,
            parent_device: device_key,
            owned: false,
        })
    }

    pub unsafe fn from_raw(device_key: device::logical::slotmap::Key, handle: vk::Image) -> Self {
        Self {
            handle,
            parent_device: device_key,
            owned: true,
        }
    }

    pub fn handle(&self) -> vk::Image {
        self.handle
    }

    pub fn parent_device(&self) -> device::logical::slotmap::Key {
        self.parent_device
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        let slotmap_device = match device::logical::slotmap::read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let device = match slotmap_device.get(self.parent_device()) {
            None => return,
            Some(value) => value,
        };
        if !self.owned {
            unsafe { device.loader().destroy_image(self.handle, None) }
        }
    }
}

pub struct ImageView {
    handle: vk::ImageView,
    parent_image: image::slotmap::Key,
}

impl ImageView {
    pub unsafe fn new(
        image_key: image::slotmap::Key,
        create_info: &vk::ImageViewCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let slotmap_image = image::slotmap::read()?;
        let image = slotmap_image
            .get(image_key)
            .ok_or_else(|| utils::make_error("image not found"))?;

        let device_key = image.parent_device();
        let slotmap_device = device::logical::slotmap::read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

        let handle = device.loader().create_image_view(create_info, None)?;
        Ok(Self {
            handle,
            parent_image: image_key,
        })
    }

    pub fn parent_image(&self) -> image::slotmap::Key {
        self.parent_image
    }

    pub fn handle(&self) -> vk::ImageView {
        self.handle
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        let slotmap_image = match image::slotmap::read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let image = match slotmap_image.get(self.parent_image()) {
            None => return,
            Some(value) => value,
        };

        let slotmap_device = match device::logical::slotmap::read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let device = match slotmap_device.get(image.parent_device()) {
            None => return,
            Some(value) => value,
        };

        unsafe { device.loader().destroy_image_view(self.handle, None) }
    }
}
