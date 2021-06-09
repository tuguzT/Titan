use std::error::Error;

use device::{Device, PhysicalDevice};
use ext::Swapchain;
use instance::Instance;
use surface::Surface;

use super::config::Config;
use super::impl_window::Window;

mod device;
mod ext;
mod instance;
mod surface;
mod utils;

pub struct Renderer {
    swapchain: Swapchain,
    device: Device,
    physical_devices: Vec<PhysicalDevice>,
    surface: Surface,
    instance: Instance,
}

impl Renderer {
    pub fn new(config: &Config, window: &Window) -> Result<Self, Box<dyn Error>> {
        let instance = Instance::new(config, window.window())?;
        log::info!(
            "instance was created! Vulkan API version is {}",
            instance.version(),
        );
        let surface = Surface::new(&instance, window.window())?;

        let mut physical_devices: Vec<PhysicalDevice> = instance
            .enumerate_physical_devices()?
            .into_iter()
            .filter(|item| {
                let iter = surface.physical_device_queue_family_properties_support(item);
                item.is_suitable()
                    && surface.is_suitable(item).unwrap_or(false)
                    && iter.peekable().peek().is_some()
            })
            .collect();
        log::info!(
            "enumerated {} suitable physical devices",
            physical_devices.len(),
        );
        physical_devices.sort_unstable();
        physical_devices.reverse();
        let best_physical_device = physical_devices
            .first()
            .ok_or_else(|| utils::make_error("no suitable physical devices were found"))?;

        let device = Device::new(&instance, &surface, best_physical_device)?;

        let swapchain =
            Swapchain::new(window, &instance, &best_physical_device, &device, &surface)?;

        Ok(Self {
            instance,
            surface,
            physical_devices,
            device,
            swapchain,
        })
    }

    pub fn render(&self) {
        log::trace!("rendering a frame!");
    }
}
