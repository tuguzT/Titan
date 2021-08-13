#![windows_subsystem = "windows"]

use std::error::Error;

use titan_engine::config::Config;
use titan_engine::window::Event;

mod logger;

const APP_NAME: &str = env!("CARGO_CRATE_NAME", "library must be compiled by Cargo");
const APP_VERSION_STR: &str = env!("CARGO_PKG_VERSION", "library must be compiled by Cargo");

fn main() -> Result<(), Box<dyn Error>> {
    let _handle = logger::init()?;
    log::info!("logger initialized successfully");

    let version = APP_VERSION_STR.parse()?;
    let config = Config::new(APP_NAME.to_string(), version);
    let application = titan_engine::init(config)?;
    application.run(move |event| match event {
        Event::Created => {
            log::debug!("created");
        }
        Event::Resized(size) => {
            let size: (u32, u32) = size.into();
            log::debug!("resized with {:?}", size);
        }
        Event::Destroyed => {
            log::debug!("destroyed");
        }
    })
}
