use std::str::FromStr;

use crate::version::Version;

#[derive(Debug)]
pub struct Config {
    name: String,
    version: Version,
}

pub const ENGINE_NAME: &'static str = env!("CARGO_CRATE_NAME", "Library must be compiled by Cargo");
pub const ENGINE_VERSION_STR: &'static str = env!("CARGO_PKG_VERSION", "Library must be compiled by Cargo");
lazy_static! {
    pub static ref ENGINE_VERSION: Version = Version::from_str(ENGINE_VERSION_STR).unwrap();
}

impl Config {
    pub fn new(name: String, version: Version) -> Self {
        Self {
            name,
            version,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }
}
