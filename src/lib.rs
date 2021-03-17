use std::error::Error;

pub mod version;
pub mod config;
mod graphics;

pub fn run(config: config::Config) -> Result<(), Box<dyn Error>> {
    let _renderer = graphics::Renderer::new(&config)?;
    Ok(())
}
