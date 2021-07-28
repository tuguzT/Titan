use std::io::Cursor;

use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use crate::error::{Error, Result};

use super::{
    device::{self, Device},
    slotmap::SlotMappable,
};

slotmap::new_key_type! {
    pub struct Key;
}

pub const VERT_SHADER_CODE: &[u8] = include_bytes!("../../res/shaders/output/vert.spv");
pub const FRAG_SHADER_CODE: &[u8] = include_bytes!("../../res/shaders/output/frag.spv");

#[derive(SlotMappable)]
pub struct ShaderModule {
    key: Key,
    handle: vk::ShaderModule,
    code: Box<[u32]>,
    parent_device: device::Key,
}

impl ShaderModule {
    pub fn new(device_key: device::Key, code: &[u8]) -> Result<Key> {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");

        let code = ash::util::read_spv(&mut Cursor::new(code))
            .map_err(|error| Error::Other {
                message: error.to_string(),
                source: Some(error.into()),
            })?
            .into_boxed_slice();
        let create_info = vk::ShaderModuleCreateInfo::builder().code(code.as_ref());
        let handle = unsafe { device.loader().create_shader_module(&create_info, None)? };

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            code,
            parent_device: device_key,
        });
        Ok(key)
    }

    pub fn handle(&self) -> vk::ShaderModule {
        self.handle
    }

    pub fn code(&self) -> &[u32] {
        self.code.as_ref()
    }

    pub fn parent_device(&self) -> device::Key {
        self.parent_device
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(self.parent_device())
            .expect("device not found");
        let loader = device.loader();
        unsafe { loader.destroy_shader_module(self.handle, None) }
    }
}
