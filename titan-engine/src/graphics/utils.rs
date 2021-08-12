#[inline]
pub const fn to_vk_version(version: &semver::Version) -> vulkano::Version {
    vulkano::Version {
        major: version.major as u32,
        minor: version.minor as u32,
        patch: version.patch as u32,
    }
}
