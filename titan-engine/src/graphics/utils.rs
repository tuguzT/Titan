use ash::vk;
use semver::Version;

#[inline]
pub const fn to_vk_version(version: &Version) -> u32 {
    vk::make_version(
        version.major as u32,
        version.minor as u32,
        version.patch as u32,
    )
}

#[inline]
pub const fn from_vk_version(version: u32) -> Version {
    Version::new(
        vk::version_major(version) as u64,
        vk::version_minor(version) as u64,
        vk::version_patch(version) as u64,
    )
}
