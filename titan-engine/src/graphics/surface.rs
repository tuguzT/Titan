use std::error::Error;

use ash::vk;
use ash_window::create_surface;
use winit::window::Window;

use proc_macro::SlotMappable;

use super::{instance, instance::Instance, slotmap::SlotMappable, utils, PhysicalDevice};

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
    pub fn new(
        key: Key,
        instance_key: instance::Key,
        window: &Window,
    ) -> Result<Self, Box<dyn Error>> {
        let slotmap = Instance::slotmap().read()?;
        let instance = slotmap
            .get(instance_key)
            .ok_or_else(|| utils::make_error("instance not found"))?;

        let loader = SurfaceLoader::new(instance.entry_loader(), instance.loader());
        let handle =
            unsafe { create_surface(instance.entry_loader(), instance.loader(), window, None) }?;
        Ok(Self {
            key,
            loader,
            handle,
            parent_instance: instance_key,
        })
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
    ) -> Result<vk::SurfaceCapabilitiesKHR, Box<dyn Error>> {
        let capabilities = unsafe {
            self.loader
                .get_physical_device_surface_capabilities(physical_device.handle(), self.handle)?
        };
        Ok(capabilities)
    }

    pub fn physical_device_formats(
        &self,
        physical_device: &PhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>, Box<dyn Error>> {
        let formats = unsafe {
            self.loader
                .get_physical_device_surface_formats(physical_device.handle(), self.handle)?
        };
        Ok(formats)
    }

    pub fn physical_device_present_modes(
        &self,
        physical_device: &PhysicalDevice,
    ) -> Result<Vec<vk::PresentModeKHR>, Box<dyn Error>> {
        let present_modes = unsafe {
            self.loader
                .get_physical_device_surface_present_modes(physical_device.handle(), self.handle)?
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
                        physical_device.handle(),
                        *index as u32,
                        self.handle,
                    )
                    .unwrap_or(false)
            })
    }

    pub fn is_suitable(&self, physical_device: &PhysicalDevice) -> Result<bool, Box<dyn Error>> {
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
