use std::error::Error;
use std::io::Cursor;

use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use super::{
    device::{self, Device},
    slotmap::SlotMappable,
    utils,
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
    code: Vec<u32>,
    parent_device: device::Key,
}

impl ShaderModule {
    pub fn new(device_key: device::Key, code: &[u8]) -> Result<Key, Box<dyn Error>> {
        let slotmap_device = Device::slotmap().read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

        let code = ash::util::read_spv(&mut Cursor::new(code))?;
        let create_info = vk::ShaderModuleCreateInfo::builder().code(code.as_slice());
        let handle = unsafe { device.loader().create_shader_module(&create_info, None)? };

        let mut slotmap = SlotMappable::slotmap().write()?;
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
        self.code.as_slice()
    }

    pub fn parent_device(&self) -> device::Key {
        self.parent_device
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        let slotmap_device = match Device::slotmap().read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let device = match slotmap_device.get(self.parent_device()) {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_shader_module(self.handle, None) }
    }
}
