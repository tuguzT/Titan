use std::ops::Deref;

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
pub struct PipelineLayout {
    #[key]
    key: Key,
    handle: vk::PipelineLayout,
    parent_device: device::Key,
}

impl HasParent<Device> for PipelineLayout {
    fn parent_key(&self) -> device::Key {
        self.parent_device
    }
}

impl HasHandle for PipelineLayout {
    type Handle = vk::PipelineLayout;

    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_> {
        Box::new(&self.handle)
    }
}

impl PipelineLayout {
    pub unsafe fn with(
        device_key: device::Key,
        create_info: &vk::PipelineLayoutCreateInfo,
    ) -> Result<Key> {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");
        let handle = device.loader().create_pipeline_layout(create_info, None)?;

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            parent_device: device_key,
        });
        Ok(key)
    }

    pub fn new(device_key: device::Key) -> Result<Key> {
        let create_info = vk::PipelineLayoutCreateInfo::default();
        unsafe { Self::with(device_key, &create_info) }
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(self.parent_key())
            .expect("device not found");
        let loader = device.loader();
        unsafe { loader.destroy_pipeline_layout(self.handle, None) }
    }
}
