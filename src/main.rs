#![windows_subsystem = "windows"]

use std::error::Error;
use std::str::FromStr;

use pretty_env_logger as logger;

use event_handler::EventHandler;
use titan_engine::config::Config;
use titan_engine::config::Version;
use titan_engine::run;

mod event_handler;

const APP_NAME: &'static str = env!("CARGO_CRATE_NAME", "Library must be compiled by Cargo");
const APP_VERSION_STR: &'static str =
    env!("CARGO_PKG_VERSION", "Library must be compiled by Cargo");

fn main() -> Result<(), Box<dyn Error>> {
    logger::try_init()?;

    let version = Version::from_str(APP_VERSION_STR)?;
    let config = Config::new(APP_NAME.to_string(), version);
    run::<EventHandler>(config)
}
