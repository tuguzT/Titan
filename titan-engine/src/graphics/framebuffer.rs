use std::error::Error;
use std::sync::{Arc, Weak};

use ash::version::DeviceV1_0;
use ash::vk;

use super::Device;

pub struct Framebuffer {
    handle: vk::Framebuffer,
    parent_device: Weak<Device>,
}

impl Framebuffer {
    pub unsafe fn new(
        device: &Arc<Device>,
        create_info: &vk::FramebufferCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let handle = device.loader().create_framebuffer(create_info, None)?;
        Ok(Self {
            handle,
            parent_device: Arc::downgrade(device),
        })
    }

    pub fn parent_device(&self) -> Option<Arc<Device>> {
        self.parent_device.upgrade()
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        let device = match self.parent_device() {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_framebuffer(self.handle, None) }
    }
}
