use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{Mutex, MutexGuard};

use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

pub use physical::PhysicalDevice;
use proc_macro::SlotMappable;
pub use queue::Queue;

use crate::error::{Error, Result};

use super::{
    ext::Swapchain,
    instance::Instance,
    slotmap::SlotMappable,
    surface::{self, Surface},
};

pub mod physical;
pub mod queue;

slotmap::new_key_type! {
    pub struct Key;
}

lazy_static::lazy_static! {
    static ref REQUIRED_EXTENSIONS: Vec<&'static CStr> = vec![Swapchain::name()];
}

#[derive(SlotMappable)]
pub struct Device {
    key: Key,
    loader: Mutex<ash::Device>,
    queue_create_infos: Vec<QueueInfo>,
    parent_physical_device: physical::Key,
}

struct QueueInfo {
    family_index: u32,
    priorities: Box<[f32]>,
}

impl Device {
    pub fn new(surface_key: surface::Key, physical_device_key: physical::Key) -> Result<Key> {
        let slotmap_surface = SlotMappable::slotmap().read().unwrap();
        let surface: &Surface = slotmap_surface.get(surface_key).expect("surface not found");
        let slotmap_physical_device = SlotMappable::slotmap().read().unwrap();
        let physical_device: &PhysicalDevice = slotmap_physical_device
            .get(physical_device_key)
            .expect("physical device not found");

        let surface_instance = surface.parent_instance();
        let physical_device_instance = physical_device.parent_instance();
        if surface_instance != physical_device_instance {
            return Err(Error::Other {
                message: String::from("surface and physical device parents must be the same"),
                source: None,
            });
        }
        let slotmap_instance = SlotMappable::slotmap().read().unwrap();
        let instance: &Instance = slotmap_instance
            .get(surface_instance)
            .expect("instance not found");

        let mut unique_family_indices = HashSet::new();
        unique_family_indices.insert(physical_device.graphics_family_index()?);
        unique_family_indices.insert(physical_device.present_family_index(surface)?);

        let priorities = [1.0];
        let queue_create_infos: Vec<_> = unique_family_indices
            .into_iter()
            .map(|family_index| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(family_index)
                    .queue_priorities(&priorities)
            })
            .collect();

        let p_layer_properties_names: Vec<*const c_char> = physical_device
            .layer_properties()
            .iter()
            .map(|item| item.layer_name.as_ptr())
            .collect();
        let p_extension_properties_names: Vec<*const c_char> = physical_device
            .extension_properties()
            .iter()
            .filter(|item| {
                let name = unsafe { CStr::from_ptr(item.extension_name.as_ptr()) };
                REQUIRED_EXTENSIONS.contains(&name)
            })
            .map(|item| item.extension_name.as_ptr())
            .collect();
        let features = vk::PhysicalDeviceFeatures::builder();
        let queue_create_infos: Vec<_> =
            queue_create_infos.iter().map(|builder| **builder).collect();
        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(queue_create_infos.as_slice())
            .enabled_layer_names(&*p_layer_properties_names)
            .enabled_extension_names(&*p_extension_properties_names)
            .enabled_features(&features);
        let loader = unsafe {
            instance.loader().instance().create_device(
                *physical_device.handle(),
                &create_info,
                None,
            )?
        };

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            queue_create_infos: queue_create_infos
                .into_iter()
                .map(|info| QueueInfo {
                    family_index: info.queue_family_index,
                    priorities: Box::from(unsafe {
                        std::slice::from_raw_parts(
                            info.p_queue_priorities,
                            info.queue_count as usize,
                        )
                    }),
                })
                .collect(),
            loader: Mutex::new(loader),
            parent_physical_device: physical_device_key,
        });
        Ok(key)
    }

    pub fn loader(&self) -> MutexGuard<ash::Device> {
        self.loader.lock().unwrap()
    }

    pub fn parent_physical_device(&self) -> physical::Key {
        self.parent_physical_device
    }

    pub fn enumerate_queues(&self) -> Result<Vec<queue::Key>> {
        let mut queues = Vec::new();
        for create_info in self.queue_create_infos.iter() {
            let vector = (0..create_info.priorities.len())
                .map(|index| unsafe {
                    Queue::new(self.key, create_info.family_index, index as u32)
                })
                .collect::<Result<Vec<_>>>()?;
            queues.extend(vector.into_iter());
        }
        Ok(queues)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.loader().destroy_device(None) };
    }
}
