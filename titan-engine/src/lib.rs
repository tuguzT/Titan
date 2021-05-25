use std::error::Error;

use config::Config;
use graphics::Renderer;
use impl_window::Window;
use window::Event;

pub mod config;
pub mod error;
pub mod window;

mod graphics;
mod impl_window;

#[cfg(feature = "jni-export")]
mod jni;

pub fn run<T>(config: Config, callback: T) -> !
where
    T: 'static + FnMut(Event),
{
    let window = handle(Window::new(&config));
    let renderer = handle(Renderer::new(&config, &window));

    window.run(renderer, callback)
}

fn handle<T>(value: Result<T, Box<dyn Error>>) -> T {
    if let Err(error) = value {
        log::error!("initialization error: {}", error);
        panic!("initialization error: {}", error);
    }
    value.unwrap()
}
