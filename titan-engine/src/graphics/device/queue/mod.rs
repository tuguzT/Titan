use std::error::Error;

use ash::version::DeviceV1_0;
use ash::vk;

use super::{super::device, utils};

pub use self::slotmap::Key;

pub mod slotmap;

pub struct Queue {
    family_index: u32,
    handle: vk::Queue,
    parent_device: device::Key,
}

impl Queue {
    pub(super) unsafe fn new(
        device_key: device::Key,
        family_index: u32,
        index: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let slotmap = device::slotmap::read()?;
        let device = slotmap
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;
        let handle = device.loader().get_device_queue(family_index, index);
        Ok(Self {
            family_index,
            handle,
            parent_device: device_key,
        })
    }

    pub fn handle(&self) -> vk::Queue {
        self.handle
    }

    pub fn parent_device(&self) -> device::Key {
        self.parent_device
    }
}
