use ash::vk;
use semver::Version;

#[macro_export]
macro_rules! c_str {
    ($s:expr) => {
        unsafe { ::std::ffi::CStr::from_ptr(crate::c_str_ptr!($s)) }
    };
}

#[macro_export]
macro_rules! c_str_ptr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const ::std::os::raw::c_char
    };
}

#[inline]
pub fn to_vk_version(version: &Version) -> u32 {
    vk::make_version(
        version.major as u32,
        version.minor as u32,
        version.patch as u32,
    )
}

#[inline]
pub fn from_vk_version(version: u32) -> Version {
    Version::new(
        vk::version_major(version) as u64,
        vk::version_minor(version) as u64,
        vk::version_patch(version) as u64,
    )
}
