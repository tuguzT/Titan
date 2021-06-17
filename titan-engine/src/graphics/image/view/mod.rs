use std::error::Error;

use ash::version::DeviceV1_0;
use ash::vk;

use super::{device, image, utils};

pub use self::slotmap::Key;

pub mod slotmap;

pub struct ImageView {
    handle: vk::ImageView,
    parent_image: image::Key,
}

impl ImageView {
    pub unsafe fn new(
        image_key: image::Key,
        create_info: &vk::ImageViewCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let slotmap_image = image::slotmap::read()?;
        let image = slotmap_image
            .get(image_key)
            .ok_or_else(|| utils::make_error("image not found"))?;

        let device_key = image.parent_device();
        let slotmap_device = device::slotmap::read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

        let handle = device.loader().create_image_view(create_info, None)?;
        Ok(Self {
            handle,
            parent_image: image_key,
        })
    }

    pub fn parent_image(&self) -> image::Key {
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

        let slotmap_device = match device::slotmap::read() {
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
