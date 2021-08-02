use std::ops::Deref;
use std::sync::Mutex;

use ash::vk;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::super::{
    device::{self, Device},
    slotmap::{HasParent, SlotMappable},
    utils::{HasHandle, HasLoader},
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct Queue {
    #[key]
    key: Key,
    family_index: u32,
    handle: Mutex<vk::Queue>,
    parent_device: device::Key,
}

impl HasParent<Device> for Queue {
    fn parent_key(&self) -> device::Key {
        self.parent_device
    }
}

impl HasHandle for Queue {
    type Handle = vk::Queue;

    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_> {
        Box::new(self.handle.lock().unwrap())
    }
}

impl Queue {
    pub(super) unsafe fn new(
        device_key: device::Key,
        family_index: u32,
        index: u32,
    ) -> Result<Key> {
        let slotmap = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap.get(device_key).expect("device not found");
        let handle = device.loader().get_device_queue(family_index, index);

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            family_index,
            handle: Mutex::new(handle),
            parent_device: device_key,
        });
        Ok(key)
    }
}
