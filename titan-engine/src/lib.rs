use std::error::Error;

use config::Config;
use graphics::Renderer;
use internal_window::Window;
use crate::window::Callback;

pub mod config;
pub mod error;
pub mod window;

mod graphics;
mod internal_window;

#[cfg(feature = "jni-export")]
mod jni;

pub fn run<T>(config: Config) -> Result<(), Box<dyn Error>>
    where T: Callback<T> + 'static
{
    let window = Window::new(&config)?;
    let renderer = Renderer::new(&config, &window)?;

    window.run::<T>(renderer);
}
