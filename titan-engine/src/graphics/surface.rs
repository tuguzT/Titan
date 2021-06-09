use std::error::Error;

use ash::vk;
use ash_window::create_surface;
use raw_window_handle::HasRawWindowHandle;

use super::Instance;
use super::PhysicalDevice;

type SurfaceLoader = ash::extensions::khr::Surface;

pub struct Surface {
    surface: vk::SurfaceKHR,
    loader: SurfaceLoader,
}

impl Surface {
    pub fn new(
        instance: &Instance,
        window_handle: &dyn HasRawWindowHandle,
    ) -> Result<Self, Box<dyn Error>> {
        let loader = SurfaceLoader::new(instance.entry_loader(), instance.loader());
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
                        self.surface,
                    )
                    .is_ok()
            })
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.loader.destroy_surface(self.surface, None) };
    }
}
