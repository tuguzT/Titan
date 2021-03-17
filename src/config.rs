use crate::version;

pub struct Config {
    pub app_name: &'static str,
    pub app_version: version::Version,
    pub engine_version: version::Version,
    pub engine_name: &'static str,
}

impl Config {
    pub fn new(app_name: &'static str, app_version: version::Version) -> Self {
        Config {
            app_name,
            app_version,
            engine_version: version::Version::new(0, 0, 1),
            engine_name: "titan",
        }
    }
}
