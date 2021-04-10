use std::error::Error;

use instance::Instance;

use crate::config::Config;
use crate::graphics::device::PhysicalDevice;

mod debug;
mod device;
mod instance;
mod utils;

pub struct Renderer {
    physical_devices: Vec<PhysicalDevice>,
    instance: Instance,
}

impl Renderer {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let instance = Instance::new(config)?;
        log::info!(
            "Instance was created! Vulkan API version is {}",
            instance.version()
        );
        let mut physical_devices: Vec<PhysicalDevice> = instance
            .enumerate_physical_devices()?
            .into_iter()
            .filter(|item| item.is_suitable())
            .collect();
        if physical_devices.is_empty() {
            return Err(Box::new(crate::error::Error::new(
                "no suitable physical devices were found".to_string(),
            )));
        }
        physical_devices.sort();
        physical_devices.reverse();

        Ok(Self {
            instance,
            physical_devices,
        })
    }
}
