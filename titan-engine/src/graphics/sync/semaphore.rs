use std::error::Error;
use std::sync::{Arc, Weak};

use ash::version::DeviceV1_0;
use ash::vk;

use crate::graphics::Device;

pub struct Semaphore {
    handle: vk::Semaphore,
    parent_device: Weak<Device>,
}

impl Semaphore {
    pub fn new(device: &Arc<Device>) -> Result<Self, Box<dyn Error>> {
        let create_info = vk::SemaphoreCreateInfo::builder();
        let handle = unsafe { device.loader().create_semaphore(&create_info, None)? };
        Ok(Self {
            handle,
            parent_device: Arc::downgrade(device),
        })
    }

    pub fn handle(&self) -> vk::Semaphore {
        self.handle
    }

    pub fn parent_device(&self) -> Option<Arc<Device>> {
        self.parent_device.upgrade()
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        let device = match self.parent_device() {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_semaphore(self.handle, None) }
    }
}
