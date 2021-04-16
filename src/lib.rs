use std::error::Error;

use config::Config;
use graphics::Renderer;
use window::Window;

pub mod config;
pub mod error;
mod graphics;
pub mod version;
mod window;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let window = Window::new(&config)?;
    let renderer = Renderer::new(&config)?;

    renderer.render();
    window.run()
}
