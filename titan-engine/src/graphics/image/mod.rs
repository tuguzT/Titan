use std::error::Error;

use ash::version::DeviceV1_0;
use ash::vk;

pub use view::ImageView;

use super::{device, image, utils};

pub use self::slotmap::Key;

pub mod slotmap;
pub mod view;

pub struct Image {
    handle: vk::Image,
    parent_device: device::Key,
    owned: bool,
}

impl Image {
    pub unsafe fn new(
        device_key: device::Key,
        create_info: &vk::ImageCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let slotmap_device = device::slotmap::read()?;
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

    pub unsafe fn from_raw(device_key: device::Key, handle: vk::Image) -> Self {
        Self {
            handle,
            parent_device: device_key,
            owned: true,
        }
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
        let slotmap_device = match device::slotmap::read() {
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
