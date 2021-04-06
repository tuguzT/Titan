#![windows_subsystem = "windows"]

use titan_rs::config::Config;
use titan_rs::version::Version;
use titan_rs::run;

fn main() {
    let config = Config::new(
        "test_name",
        Version::default(),
        cfg!(debug_assertions)
    );
    run(config).unwrap_or_else(|error| {
        eprintln!("Error is: {:#?}", error);
        std::process::exit(1)
    })
}
