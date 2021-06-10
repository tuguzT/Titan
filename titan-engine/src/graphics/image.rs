use std::error::Error;
use std::sync::{Arc, Weak};

use ash::version::DeviceV1_0;
use ash::vk;

use super::utils;
use super::Device;

pub struct Image {
    handle: vk::Image,
    parent_device: Weak<Device>,
    owned: bool,
}

impl Image {
    pub unsafe fn new(
        device: &Arc<Device>,
        create_info: &vk::ImageCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let handle = device.loader().create_image(create_info, None)?;
        Ok(Self {
            handle,
            parent_device: Arc::downgrade(device),
            owned: false,
        })
    }

    pub unsafe fn from_raw(device: &Arc<Device>, handle: vk::Image) -> Self {
        Self {
            handle,
            parent_device: Arc::downgrade(device),
            owned: true,
        }
    }

    pub fn handle(&self) -> vk::Image {
        self.handle
    }

    pub fn parent_device(&self) -> Option<Arc<Device>> {
        self.parent_device.upgrade()
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        let parent_device = match self.parent_device() {
            None => return,
            Some(value) => value,
        };
        if !self.owned {
            unsafe { parent_device.loader().destroy_image(self.handle, None) }
        }
    }
}

pub struct ImageView {
    handle: vk::ImageView,
    parent_image: Weak<Image>,
}

impl ImageView {
    pub unsafe fn new(
        image: &Arc<Image>,
        create_info: &vk::ImageViewCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let device = image
            .parent_device()
            .ok_or_else(|| utils::make_error("image parent was lost"))?;
        let handle = device.loader().create_image_view(create_info, None)?;
        Ok(Self {
            handle,
            parent_image: Arc::downgrade(image),
        })
    }

    pub fn parent_image(&self) -> Option<Arc<Image>> {
        self.parent_image.upgrade()
    }

    pub fn handle(&self) -> vk::ImageView {
        self.handle
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        let parent_image = match self.parent_image() {
            None => return,
            Some(value) => value,
        };
        let parent_device = match parent_image.parent_device() {
            None => return,
            Some(value) => value,
        };
        unsafe { parent_device.loader().destroy_image_view(self.handle, None) }
    }
}
