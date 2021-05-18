#![windows_subsystem = "windows"]

use std::error::Error;
use std::str::FromStr;

use pretty_env_logger as logger;

use titan_engine::config::Config;
use titan_engine::config::version::Version;
use titan_engine::run;

use crate::event_handler::EventHandler;

mod event_handler;

const APP_NAME: &'static str = env!("CARGO_CRATE_NAME", "Library must be compiled by Cargo");
const APP_VERSION_STR: &'static str = env!("CARGO_PKG_VERSION", "Library must be compiled by Cargo");

fn main() -> Result<(), Box<dyn Error>> {
    logger::try_init()?;

    let version = Version::from_str(APP_VERSION_STR)?;
    let config = Config::new(APP_NAME.to_string(), version);
    run::<EventHandler>(config)
}
