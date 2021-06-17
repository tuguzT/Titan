use std::error::Error;
use std::io::Cursor;

use ash::version::DeviceV1_0;
use ash::vk;

use super::{device, utils};

pub use self::slotmap::Key;

pub mod slotmap;

pub const VERT_SHADER_CODE: &[u8] = include_bytes!("../../../res/shaders/output/vert.spv");
pub const FRAG_SHADER_CODE: &[u8] = include_bytes!("../../../res/shaders/output/frag.spv");

pub struct ShaderModule {
    handle: vk::ShaderModule,
    code: Vec<u32>,
    parent_device: device::Key,
}

impl ShaderModule {
    pub fn new(device_key: device::Key, code: &[u8]) -> Result<Self, Box<dyn Error>> {
        let slotmap_device = device::slotmap::read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

        let code = ash::util::read_spv(&mut Cursor::new(code))?;
        let create_info = vk::ShaderModuleCreateInfo::builder().code(code.as_slice());
        let handle = unsafe { device.loader().create_shader_module(&create_info, None)? };
        Ok(Self {
            handle,
            code,
            parent_device: device_key,
        })
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
        let slotmap_device = match device::slotmap::read() {
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
