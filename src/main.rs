#![windows_subsystem = "windows"]

use titan_rs::config::Config;
use titan_rs::run;
use titan_rs::version::Version;

fn main() {
    let config = Config::new(
        "test_name",
        Version::default(),
        Version::default()
    );
    run(config).unwrap_or_else(|error| {
        eprintln!("Error is: {:#?}", error);
        std::process::exit(1)
    })
}
