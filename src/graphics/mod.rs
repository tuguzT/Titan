use std::error::Error;

use crate::config;

mod instance;

pub struct Renderer {
    instance: instance::Instance,
}

impl Renderer {
    pub fn new(config: &config::Config) -> Result<Self, Box<dyn Error>> {
        let instance = instance::Instance::new(config)?;
        Ok(Renderer {
            instance,
        })
    }
}
