use std::ops::Deref;
use std::sync::Mutex;

use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::super::{
    command::{self, CommandPool},
    device::Device,
    slotmap::{HasParent, SlotMappable},
    utils::{HasHandle, HasLoader},
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct CommandBuffers {
    key: Key,
    handles: Vec<Mutex<vk::CommandBuffer>>,
    parent_command_pool: command::pool::Key,
}

impl HasParent<CommandPool> for CommandBuffers {
    fn parent_key(&self) -> command::pool::Key {
        self.parent_command_pool
    }
}

impl CommandBuffers {
    pub(super) unsafe fn new(
        handles: &[vk::CommandBuffer],
        command_pool_key: command::pool::Key,
    ) -> Result<Key> {
        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handles: handles.iter().map(|handle| Mutex::new(*handle)).collect(),
            parent_command_pool: command_pool_key,
        });
        Ok(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = CommandBuffer> {
        CommandBufferIterator {
            command_buffers: &self,
            index: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.handles.len()
    }
}

struct CommandBufferIterator<'a> {
    command_buffers: &'a CommandBuffers,
    index: usize,
}

impl<'a> Iterator for CommandBufferIterator<'a> {
    type Item = CommandBuffer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.command_buffers.handles.get(index).map(|_| {
            self.index += 1;
            CommandBuffer {
                parent: &self.command_buffers,
                index,
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.command_buffers.len();
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for CommandBufferIterator<'a> {
    fn len(&self) -> usize {
        self.command_buffers.len()
    }
}

impl Drop for CommandBuffers {
    fn drop(&mut self) {
        let slotmap = SlotMappable::slotmap().read().unwrap();
        let command_pool: &CommandPool = slotmap
            .get(self.parent_key())
            .expect("command pool not found");
        let command_pool_handle = command_pool.handle();

        let slotmap = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap
            .get(command_pool.parent_key())
            .expect("device not found");

        let handles: Vec<_> = self
            .handles
            .iter()
            .map(|handle| handle.lock().unwrap())
            .collect();
        let handles: Vec<_> = handles.iter().map(|handle| **handle).collect();

        let loader = device.loader();
        unsafe { loader.free_command_buffers(**command_pool_handle, &handles) }
    }
}

pub struct CommandBuffer<'a> {
    parent: &'a CommandBuffers,
    index: usize,
}

impl<'a> HasHandle for CommandBuffer<'a> {
    type Handle = vk::CommandBuffer;

    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_> {
        let handle = &self.parent.handles[self.index];
        Box::new(handle.lock().unwrap())
    }
}

impl<'a> CommandBuffer<'a> {
    pub unsafe fn begin(&self, begin_info: &vk::CommandBufferBeginInfo) -> Result<()> {
        let slotmap_command_pool = SlotMappable::slotmap().read().unwrap();
        let command_pool: &CommandPool = slotmap_command_pool
            .get(self.parent.parent_key())
            .expect("parent was lost");

        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(command_pool.parent_key())
            .expect("command pool parent was lost");

        let loader = device.loader();
        let handle = self.handle();
        Ok(loader.begin_command_buffer(**handle, begin_info)?)
    }

    pub unsafe fn end(&self) -> Result<()> {
        let slotmap_command_pool = SlotMappable::slotmap().read().unwrap();
        let command_pool: &CommandPool = slotmap_command_pool
            .get(self.parent.parent_key())
            .expect("parent was lost");

        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(command_pool.parent_key())
            .expect("command pool parent was lost");

        let loader = device.loader();
        let handle = self.handle();
        Ok(loader.end_command_buffer(**handle)?)
    }
}
