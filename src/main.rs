#![windows_subsystem = "windows"]

use titan::config::Config;
use titan::version::Version;

fn main() {
    let config = Config::new(
        "test_name",
        Version::default(),
    );
    titan::run(config).unwrap_or_else(|error| {
        eprintln!("Error is: {:#?}", error);
        std::process::exit(1)
    })
}
