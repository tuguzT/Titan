#![windows_subsystem = "windows"]

use std::error::Error;

use titan_engine::config::Config;
use titan_engine::window::Event;

mod logger;

const APP_NAME: &str = env!("CARGO_CRATE_NAME", "library must be compiled by Cargo");
const APP_VERSION_STR: &str = env!("CARGO_PKG_VERSION", "library must be compiled by Cargo");

fn main() -> Result<(), Box<dyn Error>> {
    let _handle = logger::init().unwrap();
    log::info!("logger initialized successfully");

    let version = APP_VERSION_STR.parse().unwrap();
    let enable_validation = cfg!(debug_assertions);
    let config = Config::new(APP_NAME.to_string(), version, enable_validation);

    let application = titan_engine::init(config)?;
    application.run(move |event| match event {
        Event::Created => {
            log::debug!("created");
        }
        Event::Resized(size) => {
            let size: (u32, u32) = size.into();
            log::debug!("resized with {:?}", size);
        }
        Event::Update(delta_time) => {
            log::debug!("delta time: {:?}", delta_time);
        }
        Event::Destroyed => {
            log::debug!("destroyed");
        }
    })
}
