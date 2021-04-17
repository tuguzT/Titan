use std::error::Error;

use ash::vk;
use ash_window::create_surface;
use raw_window_handle::HasRawWindowHandle;

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
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.loader.destroy_surface(self.surface, None) };
    }
}
