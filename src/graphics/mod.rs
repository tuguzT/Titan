use std::error::Error;

use instance::Instance;

use crate::config::Config;
use crate::graphics::device::PhysicalDevice;

mod utils;
mod instance;
mod device;
mod debug;

pub struct Renderer {
    physical_devices: Vec<PhysicalDevice>,
    instance: Instance,
}

impl Renderer {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let instance = Instance::new(config)?;
        log::info!("Instance was created! Vulkan API version is {}", instance.version());
        let physical_devices = instance.enumerate_physical_devices()?;

        Ok(Self {
            instance,
            physical_devices,
        })
    }
}
