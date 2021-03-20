use std::error::Error;

use instance::Instance;

use crate::config::Config;

mod utils;
mod instance;

pub struct Renderer {
    instance: Instance,
}

impl Renderer {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let instance = Instance::new(config)?;
        println!("Instance was created! {:#?}", instance.version());

        Ok(Self {
            instance,
        })
    }
}
