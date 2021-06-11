use std::error::Error;
use std::sync::{Arc, Weak};

use ash::version::DeviceV1_0;
use ash::vk;

use super::utils;
use super::Device;

pub struct CommandPool {
    handle: vk::CommandPool,
    parent_device: Weak<Device>,
}

impl CommandPool {
    pub unsafe fn new(
        device: &Arc<Device>,
        create_info: &vk::CommandPoolCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let handle = device.loader().create_command_pool(create_info, None)?;
        Ok(Self {
            handle,
            parent_device: Arc::downgrade(device),
        })
    }

    pub fn handle(&self) -> vk::CommandPool {
        self.handle
    }

    pub fn parent_device(&self) -> Option<Arc<Device>> {
        self.parent_device.upgrade()
    }

    pub fn enumerate_command_buffers(
        this: &Arc<Self>,
        count: u32,
    ) -> Result<Vec<CommandBuffer>, Box<dyn Error>> {
        let device = this
            .parent_device()
            .ok_or_else(|| utils::make_error("parent was lost"))?;
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(this.handle())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);
        Ok(unsafe {
            device
                .loader()
                .allocate_command_buffers(&allocate_info)?
                .into_iter()
                .map(|command_buffer| CommandBuffer::new(this, command_buffer))
                .collect()
        })
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        let device = match self.parent_device() {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_command_pool(self.handle, None) }
    }
}

pub struct CommandBuffer {
    handle: vk::CommandBuffer,
    parent_command_pool: Weak<CommandPool>,
}

impl CommandBuffer {
    unsafe fn new(command_pool: &Arc<CommandPool>, handle: vk::CommandBuffer) -> Self {
        Self {
            handle,
            parent_command_pool: Arc::downgrade(command_pool),
        }
    }

    pub fn parent_command_pool(&self) -> Option<Arc<CommandPool>> {
        self.parent_command_pool.upgrade()
    }

    pub fn handle(&self) -> vk::CommandBuffer {
        self.handle
    }

    pub unsafe fn begin(
        &self,
        begin_info: &vk::CommandBufferBeginInfo,
    ) -> Result<(), Box<dyn Error>> {
        let command_pool = self
            .parent_command_pool()
            .ok_or_else(|| utils::make_error("parent was lost"))?;
        let device = command_pool
            .parent_device()
            .ok_or_else(|| utils::make_error("command pool parent was lost"))?;
        Ok(device
            .loader()
            .begin_command_buffer(self.handle, begin_info)?)
    }

    pub unsafe fn end(&self) -> Result<(), Box<dyn Error>> {
        let command_pool = self
            .parent_command_pool()
            .ok_or_else(|| utils::make_error("parent was lost"))?;
        let device = command_pool
            .parent_device()
            .ok_or_else(|| utils::make_error("command pool parent was lost"))?;
        Ok(device.loader().end_command_buffer(self.handle)?)
    }
}
