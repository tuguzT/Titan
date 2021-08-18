use std::sync::Arc;

use vulkano::instance::{ApplicationInfo, Instance, InstanceCreationError};
use vulkano_win::required_extensions;

use crate::config::{Config, ENGINE_NAME, ENGINE_VERSION};
use crate::error::{Error, Result};

#[inline]
const fn to_vk_version(version: &semver::Version) -> vulkano::Version {
    vulkano::Version {
        major: version.major as u32,
        minor: version.minor as u32,
        patch: version.patch as u32,
    }
}

pub fn create_instance(config: &Config) -> Result<Arc<Instance>> {
    let info = ApplicationInfo {
        application_name: Some(config.name().into()),
        application_version: Some(self::to_vk_version(config.version())),
        engine_name: Some(ENGINE_NAME.into()),
        engine_version: Some(self::to_vk_version(&*ENGINE_VERSION)),
    };
    let extensions = {
        let mut extensions = self::required_extensions();
        if config.enable_validation() {
            extensions.ext_debug_utils = true;
        }
        extensions
    };
    let layers = config
        .enable_validation()
        .then(|| "VK_LAYER_KHRONOS_validation");

    let instance = Instance::new(Some(&info), vulkano::Version::V1_2, &extensions, layers)?;
    Ok(instance)
}

impl From<InstanceCreationError> for Error {
    fn from(error: InstanceCreationError) -> Self {
        Self::new("instance creation failure", error)
    }
}
