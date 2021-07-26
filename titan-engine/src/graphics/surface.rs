use ash::vk;
use winit::window::Window;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::{
    instance::{self, Instance},
    slotmap::SlotMappable,
    PhysicalDevice,
};

slotmap::new_key_type! {
    pub struct Key;
}

type SurfaceLoader = ash::extensions::khr::Surface;

#[derive(SlotMappable)]
pub struct Surface {
    key: Key,
    handle: vk::SurfaceKHR,
    loader: SurfaceLoader,
    parent_instance: instance::Key,
}

impl Surface {
    pub fn new(instance_key: instance::Key, window: &Window) -> Result<Key> {
        let slotmap = SlotMappable::slotmap().read().unwrap();
        let instance: &Instance = slotmap.get(instance_key).expect("instance not found");

        let instance_loader = instance.loader();
        let loader = SurfaceLoader::new(instance_loader.entry(), instance_loader.instance());
        let handle = unsafe {
            ash_window::create_surface(
                instance_loader.entry(),
                instance_loader.instance(),
                window,
                None,
            )?
        };

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            loader,
            handle,
            parent_instance: instance_key,
        });
        Ok(key)
    }

    pub fn handle(&self) -> vk::SurfaceKHR {
        self.handle
    }

    pub fn parent_instance(&self) -> instance::Key {
        self.parent_instance
    }

    pub fn physical_device_capabilities(
        &self,
        physical_device: &PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR> {
        let capabilities = unsafe {
            self.loader
                .get_physical_device_surface_capabilities(*physical_device.handle(), self.handle)?
        };
        Ok(capabilities)
    }

    pub fn physical_device_formats(
        &self,
        physical_device: &PhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>> {
        let formats = unsafe {
            self.loader
                .get_physical_device_surface_formats(*physical_device.handle(), self.handle)?
        };
        Ok(formats)
    }

    pub fn physical_device_present_modes(
        &self,
        physical_device: &PhysicalDevice,
    ) -> Result<Vec<vk::PresentModeKHR>> {
        let present_modes = unsafe {
            self.loader
                .get_physical_device_surface_present_modes(*physical_device.handle(), self.handle)?
        };
        Ok(present_modes)
    }

    pub fn physical_device_queue_family_properties_support<'a>(
        &'a self,
        physical_device: &'a PhysicalDevice,
    ) -> impl Iterator<Item = (usize, &'a vk::QueueFamilyProperties)> {
        physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .filter(move |(index, _queue_family_properties)| unsafe {
                self.loader
                    .get_physical_device_surface_support(
                        *physical_device.handle(),
                        *index as u32,
                        self.handle,
                    )
                    .unwrap_or(false)
            })
    }

    pub fn is_suitable(&self, physical_device: &PhysicalDevice) -> Result<bool> {
        let formats = self.physical_device_formats(physical_device)?;
        let present_modes = self.physical_device_present_modes(physical_device)?;
        Ok(!formats.is_empty() && !present_modes.is_empty())
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.loader.destroy_surface(self.handle, None) };
    }
}
