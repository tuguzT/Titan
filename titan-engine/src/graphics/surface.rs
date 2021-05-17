use std::error::Error;

use ash::vk;
use ash_window::create_surface;
use raw_window_handle::HasRawWindowHandle;

use crate::graphics::device::PhysicalDevice;
use crate::graphics::instance::Instance;

pub struct Surface {
    surface: vk::SurfaceKHR,
    loader: ash::extensions::khr::Surface,
}

impl Surface {
    pub fn new(
        instance: &Instance,
        window_handle: &dyn HasRawWindowHandle,
    ) -> Result<Self, Box<dyn Error>> {
        use ash::extensions::khr::Surface;

        let loader = Surface::new(instance.entry_loader(), instance.loader());
        let surface = unsafe {
            create_surface(
                instance.entry_loader(),
                instance.loader(),
                window_handle,
                None,
            )
        }?;
        Ok(Self { loader, surface })
    }

    pub fn physical_device_queue_family_properties_support<'a, 'b>(
        &'a self,
        physical_device: &'b PhysicalDevice,
    ) -> Result<Vec<(usize, &'b vk::QueueFamilyProperties)>, Box<dyn Error>> {
        let mut vector = Vec::with_capacity(physical_device.queue_family_properties().len());
        for (index, queue_family_properties) in
            physical_device.queue_family_properties().iter().enumerate()
        {
            let supported = unsafe {
                self.loader.get_physical_device_surface_support(
                    physical_device.handle(),
                    index as u32,
                    self.surface,
                )
            }?;
            if supported {
                vector.push((index, queue_family_properties));
            }
        }
        Ok(vector)
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.loader.destroy_surface(self.surface, None) };
    }
}
