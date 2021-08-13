use semver::Version;

#[derive(Debug, Clone)]
pub struct Config {
    name: String,
    version: Version,
}

pub const ENGINE_NAME: &str = env!("CARGO_CRATE_NAME", "library must be compiled by Cargo");

const ENGINE_VERSION_STR: &str = env!("CARGO_PKG_VERSION", "library must be compiled by Cargo");
lazy_static::lazy_static! {
    pub static ref ENGINE_VERSION: Version = ENGINE_VERSION_STR.parse().unwrap();
}

impl Config {
    pub const fn new(name: String, version: Version) -> Self {
        Self { name, version }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new("Hello World".to_string(), Version::new(0, 0, 0))
    }
}
