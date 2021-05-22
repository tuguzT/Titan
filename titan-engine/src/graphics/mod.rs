use std::error::Error;

use device::{Device, PhysicalDevice};
use instance::Instance;

use crate::config::Config;
use crate::graphics::surface::Surface;
use crate::impl_window::Window;

mod debug;
mod device;
mod instance;
mod surface;
mod utils;

pub struct Renderer {
    device: Device,
    physical_devices: Vec<PhysicalDevice>,
    surface: Surface,
    instance: Instance,
}

impl Renderer {
    pub fn new(config: &Config, window: &Window) -> Result<Self, Box<dyn Error>> {
        use crate::error::{Error, ErrorType};

        let instance = Instance::new(config, window.window())?;
        log::info!(
            "Instance was created! Vulkan API version is {}",
            instance.version()
        );
        let surface = Surface::new(&instance, window.window())?;

        let mut physical_devices: Vec<PhysicalDevice> = instance
            .enumerate_physical_devices()?
            .into_iter()
            .filter(
                |item| match surface.physical_device_queue_family_properties_support(item) {
                    Ok(vector) => item.is_suitable() && !vector.is_empty(),
                    Err(_) => false,
                },
            )
            .collect();
        log::info!(
            "Enumerated {} suitable physical devices",
            physical_devices.len()
        );
        if physical_devices.is_empty() {
            return Err(Error::new(
                "no suitable physical devices were found",
                ErrorType::Graphics,
            ).into());
        }
        physical_devices.sort_unstable();
        physical_devices.reverse();
        let best_physical_device = physical_devices.first().unwrap();
        let device = Device::new(&instance, best_physical_device)?;

        Ok(Self {
            instance,
            surface,
            physical_devices,
            device,
        })
    }

    pub fn render(&self) {
        log::debug!("Rendering a frame!");
    }
}
