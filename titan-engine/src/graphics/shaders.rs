use std::error::Error;
use std::sync::{Weak, Arc};

use ash::vk;
use ash::version::DeviceV1_0;

use super::Device;

pub const VERT_SHADER_CODE: &[u8] = include_bytes!("../../res/shaders/output/vert.spv");
pub const FRAG_SHADER_CODE: &[u8] = include_bytes!("../../res/shaders/output/frag.spv");

pub struct ShaderModule {
    handle: vk::ShaderModule,
    code: Vec<u32>,
    parent_device: Weak<Device>,
}

impl ShaderModule {
    pub fn new(device: &Arc<Device>, code: &[u8]) -> Result<Self, Box<dyn Error>> {
        let code = unsafe { code.align_to().1.to_owned() };
        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(code.as_slice());
        let handle = unsafe {
            device.loader().create_shader_module(&create_info, None)?
        };
        Ok(Self {
            handle,
            code,
            parent_device: Arc::downgrade(device),
        })
    }

    pub fn parent_device(&self) -> Option<Arc<Device>> {
        self.parent_device.upgrade()
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        let device = match self.parent_device() {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_shader_module(self.handle, None) }
    }
}
