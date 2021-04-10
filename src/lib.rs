use std::error::Error;

use graphics::Renderer;

use crate::config::Config;

pub mod config;
pub mod error;
mod graphics;
pub mod version;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let _renderer = Renderer::new(&config)?;
    Ok(())
}
