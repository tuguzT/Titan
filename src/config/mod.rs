//! Configuration utilities for game engine and your game.

use semver::Version;

/// This struct represents general configuration of game engine.
#[derive(Debug, Clone)]
pub struct Config {
    name: String,
    version: Version,
    enable_validation: bool,
}

pub const ENGINE_NAME: &str = env!("CARGO_CRATE_NAME", "library must be compiled by Cargo");

const ENGINE_VERSION_STR: &str = env!("CARGO_PKG_VERSION", "library must be compiled by Cargo");
lazy_static::lazy_static! {
    pub static ref ENGINE_VERSION: Version = ENGINE_VERSION_STR.parse().unwrap();
}

impl Config {
    /// Creates new configuration with given name, version and validation usage.
    pub const fn new(name: String, version: Version, enable_validation: bool) -> Self {
        Self {
            name,
            version,
            enable_validation,
        }
    }

    /// Name of your game.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Semver version of your game.
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// If game will use validation (useful for debugging).
    pub fn enable_validation(&self) -> bool {
        self.enable_validation
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(
            "Hello World".to_string(),
            Version::new(0, 0, 0),
            cfg!(debug_assertions),
        )
    }
}
