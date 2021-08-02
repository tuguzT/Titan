use std::ops::Deref;

use ash::vk;
use semver::Version;

pub trait HasHandle {
    type Handle;

    // FIXME: change to `impl Deref<...>` if stable
    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_>;
}

pub trait HasLoader {
    type Loader;

    // FIXME: change to `impl Deref<...>` if stable
    fn loader(&self) -> Box<dyn Deref<Target = Self::Loader> + '_>;
}

#[inline]
pub const fn to_vk_version(version: &Version) -> u32 {
    vk::make_api_version(
        0,
        version.major as u32,
        version.minor as u32,
        version.patch as u32,
    )
}

#[inline]
pub const fn from_vk_version(version: u32) -> Version {
    Version::new(
        vk::api_version_major(version) as u64,
        vk::api_version_minor(version) as u64,
        vk::api_version_patch(version) as u64,
    )
}
