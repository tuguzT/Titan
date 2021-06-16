use std::error::Error;

use ash::version::DeviceV1_0;
use ash::vk;

use super::super::{device, utils};

pub mod slotmap;

pub struct Fence {
    handle: vk::Fence,
    parent_device: device::logical::slotmap::Key,
}

impl Fence {
    pub fn new(
        device_key: device::logical::slotmap::Key,
        create_info: &vk::FenceCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let slotmap_device = device::logical::slotmap::read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

        let handle = unsafe { device.loader().create_fence(&create_info, None)? };
        Ok(Self {
            handle,
            parent_device: device_key,
        })
    }

    pub fn handle(&self) -> vk::Fence {
        self.handle
    }

    pub fn parent_device(&self) -> device::logical::slotmap::Key {
        self.parent_device
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        let slotmap_device = match device::logical::slotmap::read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let device = match slotmap_device.get(self.parent_device()) {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_fence(self.handle, None) }
    }
}
