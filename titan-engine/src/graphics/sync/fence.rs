use std::error::Error;
use std::sync::{Arc, Weak};

use ash::version::DeviceV1_0;
use ash::vk;

use crate::graphics::Device;

pub struct Fence {
    handle: vk::Fence,
    parent_device: Weak<Device>,
}

impl Fence {
    pub fn new(
        device: &Arc<Device>,
        create_info: &vk::FenceCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let handle = unsafe { device.loader().create_fence(&create_info, None)? };
        Ok(Self {
            handle,
            parent_device: Arc::downgrade(device),
        })
    }

    pub fn handle(&self) -> vk::Fence {
        self.handle
    }

    pub fn parent_device(&self) -> Option<Arc<Device>> {
        self.parent_device.upgrade()
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        let device = match self.parent_device() {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_fence(self.handle, None) }
    }
}
