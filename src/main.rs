#![windows_subsystem = "windows"]

use std::error::Error;

use titan_rs::config::Config;
use titan_rs::run;
use titan_rs::version::Version;

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::try_init()?;

    let config = Config::new("test-app".to_string(), Version::default());
    run(config)
}
