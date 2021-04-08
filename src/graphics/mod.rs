use std::error::Error;

use instance::Instance;

use crate::config::Config;

mod utils;
mod instance;
mod device;
mod debug;

pub struct Renderer {
    instance: Instance,
}

impl Renderer {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let instance = Instance::new(config)?;
        log::info!("Instance was created! Vulkan API version is {}", instance.version());
        let _physical_devices = instance.enumerate_physical_devices()?;

        Ok(Self {
            instance,
        })
    }
}
