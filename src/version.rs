#[derive(Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl std::default::Default for Version {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
        }
    }
}
