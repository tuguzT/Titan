use std::ffi::CString;

use crate::version;

pub struct Config {
    app_name: &'static str,
    app_name_c: CString,
    app_version: version::Version,
    engine_name: &'static str,
    engine_name_c: CString,
    engine_version: version::Version,
}

impl Config {
    pub fn new(app_name: &'static str, app_version: version::Version) -> Self {
        let engine_name = "titan";
        Self {
            app_name,
            app_version,
            engine_name,
            engine_version: version::Version::default(),
            app_name_c: CString::new(app_name).unwrap(),
            engine_name_c: CString::new(engine_name).unwrap(),
        }
    }

    pub fn app_name(&self) -> &'static str {
        self.app_name
    }

    pub fn app_version(&self) -> &version::Version {
        &self.app_version
    }

    pub fn engine_name(&self) -> &'static str {
        self.engine_name
    }

    pub fn engine_version(&self) -> &version::Version {
        &self.engine_version
    }

    pub fn app_name_c(&self) -> &CString {
        &self.app_name_c
    }

    pub fn engine_name_c(&self) -> &CString {
        &self.engine_name_c
    }
}
