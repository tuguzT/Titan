#![windows_subsystem = "windows"]

use std::error::Error;
use std::str::FromStr;

use titan_rs::config::Config;
use titan_rs::run;
use titan_rs::version::Version;

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::try_init()?;

    let config_bytes = include_bytes!("../res/config.json");
    let config = parse_config(config_bytes)?;
    run(config)
}

fn parse_config(bytes: &[u8]) -> Result<Config, Box<dyn Error>> {
    let json: serde_json::Value = serde_json::from_slice(bytes)?;

    let name = json["name"].as_str()
        .expect(r#"config.json must contain "name" string"#);
    let version = json["version"].as_str()
        .expect(r#"config.json must contain "version" semver string"#);
    let version = Version::from_str(version)?;

    Ok(Config::new(name.to_string(), version))
}
