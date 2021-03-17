#![windows_subsystem = "windows"]

use titan::{config, version};

fn main() {
    let config = config::Config::new(
        "test_name",
        version::Version::default(),
    );
    titan::run(config).unwrap_or_else(|error| {
        eprintln!("Error is: {:#?}", error);
        std::process::exit(1)
    })
}
