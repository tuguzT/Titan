use crate::version::Version;

#[derive(Debug)]
pub struct Config {
    name: String,
    version: Version,
}

pub const ENGINE_NAME: &'static str = "titan";
lazy_static! {
    pub static ref ENGINE_VERSION: Version = Version::default();
}

impl Config {
    pub fn new(name: String, version: Version) -> Self {
        Self {
            name,
            version,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }
}
