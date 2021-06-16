use std::error::Error;

use ash::version::DeviceV1_0;
use ash::vk;

use super::{command, device, utils};

pub mod buffer;
pub mod pool;

pub struct CommandPool {
    key: command::pool::slotmap::Key,
    handle: vk::CommandPool,
    parent_device: device::logical::slotmap::Key,
}

impl CommandPool {
    pub unsafe fn new(
        key: command::pool::slotmap::Key,
        device_key: device::logical::slotmap::Key,
        create_info: &vk::CommandPoolCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let slotmap_device = device::logical::slotmap::read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

        let handle = device.loader().create_command_pool(create_info, None)?;
        Ok(Self {
            key,
            handle,
            parent_device: device_key,
        })
    }

    pub fn handle(&self) -> vk::CommandPool {
        self.handle
    }

    pub fn parent_device(&self) -> device::logical::slotmap::Key {
        self.parent_device
    }

    pub fn enumerate_command_buffers(
        &self,
        count: u32,
    ) -> Result<Vec<CommandBuffer>, Box<dyn Error>> {
        let device_key = self.parent_device();
        let slotmap_device = device::logical::slotmap::read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("parent was lost"))?;

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.handle())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);
        Ok(unsafe {
            device
                .loader()
                .allocate_command_buffers(&allocate_info)?
                .into_iter()
                .map(|command_buffer| CommandBuffer::new(self.key, command_buffer))
                .collect()
        })
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        let slotmap_device = device::logical::slotmap::read();
        let slotmap_device = match slotmap_device {
            Ok(value) => value,
            Err(_) => return,
        };
        let device = match slotmap_device.get(self.parent_device()) {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_command_pool(self.handle, None) }
    }
}

pub struct CommandBuffer {
    handle: vk::CommandBuffer,
    parent_command_pool: command::pool::slotmap::Key,
}

impl CommandBuffer {
    unsafe fn new(
        command_pool_key: command::pool::slotmap::Key,
        handle: vk::CommandBuffer,
    ) -> Self {
        Self {
            handle,
            parent_command_pool: command_pool_key,
        }
    }

    pub fn parent_command_pool(&self) -> command::pool::slotmap::Key {
        self.parent_command_pool
    }

    pub fn handle(&self) -> vk::CommandBuffer {
        self.handle
    }

    pub unsafe fn begin(
        &self,
        begin_info: &vk::CommandBufferBeginInfo,
    ) -> Result<(), Box<dyn Error>> {
        let slotmap_command_pool = command::pool::slotmap::read()?;
        let command_pool = slotmap_command_pool
            .get(self.parent_command_pool())
            .ok_or_else(|| utils::make_error("parent was lost"))?;

        let slotmap_device = device::logical::slotmap::read()?;
        let device = slotmap_device
            .get(command_pool.parent_device())
            .ok_or_else(|| utils::make_error("command pool parent was lost"))?;

        Ok(device
            .loader()
            .begin_command_buffer(self.handle, begin_info)?)
    }

    pub unsafe fn end(&self) -> Result<(), Box<dyn Error>> {
        let slotmap_command_pool = command::pool::slotmap::read()?;
        let command_pool = slotmap_command_pool
            .get(self.parent_command_pool())
            .ok_or_else(|| utils::make_error("parent was lost"))?;

        let slotmap_device = device::logical::slotmap::read()?;
        let device = slotmap_device
            .get(command_pool.parent_device())
            .ok_or_else(|| utils::make_error("command pool parent was lost"))?;

        Ok(device.loader().end_command_buffer(self.handle)?)
    }
}
