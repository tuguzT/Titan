use std::error::Error;

use ash::version::EntryV1_0;
use ash::vk;

use crate::config::Config;
use crate::version::Version;

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
        let version = match lib_entry.try_enumerate_instance_version() {
            Ok(version) => {
                from_vk_version(version.unwrap_or(vk::API_VERSION_1_0))
            },
            Err(_) => Version::default()
        };
        let layer_properties = lib_entry
            .enumerate_instance_layer_properties()
            .unwrap_or(Vec::new());
        let extension_properties = lib_entry
            .enumerate_instance_extension_properties()
            .unwrap_or(Vec::new());

        let application_version = to_vk_version(&config.app_version());
        let engine_version = to_vk_version(&config.engine_version());
        let application_info = vk::ApplicationInfo {
            p_application_name: config.app_name_c().as_ptr(),
            application_version,
            p_engine_name: config.engine_name_c().as_ptr(),
            engine_version,
            api_version: vk::API_VERSION_1_2,
            ..Default::default()
        };
        vk::ApplicationInfo::builder().build();
        let instance_create_info = vk::InstanceCreateInfo {
            p_application_info: &application_info,
            ..Default::default()
        };
        let instance = unsafe {
            lib_entry.create_instance(&instance_create_info, None)?
        };
        println!("Instance was created! {:#?}", version);
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

    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn lib_entry(&self) -> &ash::Entry {
        &self.lib_entry
    }

    pub fn extension_properties(&self) -> &Vec<vk::ExtensionProperties> {
        &self.extension_properties
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
