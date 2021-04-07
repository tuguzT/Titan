use std::error::Error;

use instance::Instance;

use crate::config::Config;

mod utils;
mod instance;
mod device;

pub struct Renderer {
    instance: Instance,
}

impl Renderer {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let instance = Instance::new(config)?;
        log::info!("Instance was created! Vulkan API version is {}", instance.version());

        Ok(Self {
            instance,
        })
    }
}
