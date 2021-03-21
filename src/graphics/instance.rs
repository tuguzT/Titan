use std::error::Error;
use std::ffi::CString;

use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;

use crate::config::Config;
use crate::version::Version;

const VALIDATION_LAYERS: [&str; 1] = [
    "VK_LAYER_KHRONOS_validation",
];

pub struct Instance {
    version: Version,
    layer_properties: Vec<vk::LayerProperties>,
    extension_properties: Vec<vk::ExtensionProperties>,
    instance: ash::Instance,
    lib_entry: ash::Entry,
}

impl Instance {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let lib_entry = unsafe {
            ash::Entry::new()?
        };
        let version = match lib_entry.try_enumerate_instance_version()? {
            Some(version) => from_vk_version(version),
            None => from_vk_version(vk::API_VERSION_1_0),
        };
        let layer_properties = lib_entry
            .enumerate_instance_layer_properties()?;
        let extension_properties = lib_entry
            .enumerate_instance_extension_properties()?;

        let application_name = CString::new(config.app_name())?;
        let engine_name = CString::new(config.engine_name())?;
        let application_info = vk::ApplicationInfo {
            application_version: to_vk_version(&config.app_version()),
            engine_version: to_vk_version(&config.engine_version()),
            p_application_name: application_name.as_ptr(),
            p_engine_name: engine_name.as_ptr(),
            api_version: vk::API_VERSION_1_2,
            ..Default::default()
        };

        let instance_create_info = vk::InstanceCreateInfo {
            p_application_info: &application_info,
            ..Default::default()
        };
        let instance = unsafe {
            lib_entry.create_instance(&instance_create_info, None)?
        };

        Ok(Self {
            instance,
            version,
            lib_entry,
            layer_properties,
            extension_properties,
        })
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn layer_properties(&self) -> &Vec<vk::LayerProperties> {
        &self.layer_properties
    }

    pub fn extension_properties(&self) -> &Vec<vk::ExtensionProperties> {
        &self.extension_properties
    }

    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn lib_entry(&self) -> &ash::Entry {
        &self.lib_entry
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

#[inline]
fn to_vk_version(version: &Version) -> u32 {
    vk::make_version(
        version.major,
        version.minor,
        version.patch,
    )
}

#[inline]
fn from_vk_version(version: u32) -> Version {
    Version::new(
        vk::version_major(version),
        vk::version_minor(version),
        vk::version_patch(version),
    )
}
