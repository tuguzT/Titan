use std::error::Error;

use config::Config;
use graphics::Renderer;
use window::Window;

pub mod config;
pub mod error;
mod graphics;
pub mod version;
mod window;

#[cfg(feature = "jni-export")]
mod jni;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let window = Window::new(&config)?;
    let renderer = Renderer::new(&config, &window)?;

    window.run(renderer);
}
