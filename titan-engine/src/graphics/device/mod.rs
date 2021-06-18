use std::collections::HashSet;
use std::error::Error;
use std::ffi::CStr;
use std::ops::Deref;
use std::os::raw::c_char;

use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

pub use physical::PhysicalDevice;
use proc_macro::SlotMappable;
pub use queue::Queue;

use super::{
    ext::Swapchain, instance::Instance, slotmap::SlotMappable, surface, surface::Surface, utils,
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
    loader: ash::Device,
    queue_create_infos: Vec<vk::DeviceQueueCreateInfo>,
    parent_physical_device: physical::Key,
}

unsafe impl Send for Device {}

unsafe impl Sync for Device {}

impl Device {
    pub fn new(
        surface_key: surface::Key,
        physical_device_key: physical::Key,
    ) -> Result<Key, Box<dyn Error>> {
        let slotmap_surface = Surface::slotmap().read()?;
        let surface = slotmap_surface
            .get(surface_key)
            .ok_or_else(|| utils::make_error("surface not found"))?;
        let slotmap_physical_device = PhysicalDevice::slotmap().read()?;
        let physical_device = slotmap_physical_device
            .get(physical_device_key)
            .ok_or_else(|| utils::make_error("physical device not found"))?;

        let surface_instance = surface.parent_instance();
        let physical_device_instance = physical_device.parent_instance();
        if surface_instance != physical_device_instance {
            return Err(
                utils::make_error("surface and physical device parents must be the same").into(),
            );
        }
        let slotmap_instance = Instance::slotmap().read()?;
        let instance = slotmap_instance
            .get(surface_instance)
            .ok_or_else(|| utils::make_error("instance not found"))?;

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
                    .build()
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
        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(queue_create_infos.as_slice())
            .enabled_layer_names(p_layer_properties_names.deref())
            .enabled_extension_names(p_extension_properties_names.deref())
            .enabled_features(&features);
        let loader = unsafe {
            instance
                .loader()
                .create_device(physical_device.handle(), &create_info, None)?
        };

        let mut slotmap = SlotMappable::slotmap().write()?;
        let key = slotmap.insert_with_key(|key| Self {
            key,
            queue_create_infos,
            loader,
            parent_physical_device: physical_device_key,
        });
        Ok(key)
    }

    pub fn loader(&self) -> &ash::Device {
        &self.loader
    }

    pub fn handle(&self) -> vk::Device {
        self.loader.handle()
    }

    pub fn parent_physical_device(&self) -> physical::Key {
        self.parent_physical_device
    }

    pub fn enumerate_queues(&self) -> Result<Vec<queue::Key>, Box<dyn Error>> {
        let mut queues = Vec::new();
        for create_info in self.queue_create_infos.iter() {
            let range = 0..create_info.queue_count;
            let vector: Result<Vec<_>, _> = range
                .map(|index| unsafe { Queue::new(self.key, create_info.queue_family_index, index) })
                .collect();
            vector.map(|vector| queues.extend(vector.into_iter()))?
        }
        Ok(queues)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.loader.destroy_device(None) };
    }
}
