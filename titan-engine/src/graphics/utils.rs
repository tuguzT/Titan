use ash::vk;

use crate::config::Version;
use crate::error::{Error, ErrorType};

#[macro_export]
macro_rules! c_str {
    ($s:expr) => {
        unsafe { std::ffi::CStr::from_ptr(crate::c_str_ptr!($s)) }
    };
}

#[macro_export]
macro_rules! c_str_ptr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const std::os::raw::c_char
    };
}

#[inline]
pub fn to_vk_version(version: &Version) -> u32 {
    vk::make_version(version.major, version.minor, version.patch)
}

#[inline]
pub fn from_vk_version(version: u32) -> Version {
    Version::new(
        vk::version_major(version),
        vk::version_minor(version),
        vk::version_patch(version),
        "".to_string(),
    )
}

#[inline]
pub fn make_error(message: &'static str) -> Error {
    Error::new(message, ErrorType::Graphics)
}
