pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Version {
            major,
            minor,
            patch,
        }
    }
}

impl std::default::Default for Version {
    fn default() -> Self {
        Version {
            major: 0,
            minor: 0,
            patch: 0,
        }
    }
}
