use std::error::Error;
use std::ffi::CString;

use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;

use crate::config::Config;
use crate::version::Version;
use crate::graphics::utils;
use std::os::raw::c_char;

const VALIDATION_LAYER_NAME: &str = "VK_LAYER_KHRONOS_validation";

pub struct Instance {
    version: Version,
    available_layer_properties: Vec<vk::LayerProperties>,
    available_extension_properties: Vec<vk::ExtensionProperties>,
    instance: ash::Instance,
    lib_entry: ash::Entry,
}

impl Instance {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let lib_entry = unsafe {
            ash::Entry::new()?
        };
        let version = match lib_entry.try_enumerate_instance_version()? {
            Some(version) => utils::from_vk_version(version),
            None => utils::from_vk_version(vk::API_VERSION_1_0),
        };
        let available_layer_properties = lib_entry
            .enumerate_instance_layer_properties()?;
        let available_extension_properties = lib_entry
            .enumerate_instance_extension_properties()?;

        let application_name = CString::new(config.app_name())?;
        let engine_name = CString::new(config.engine_name())?;
        let application_info = vk::ApplicationInfo {
            application_version: utils::to_vk_version(&config.app_version()),
            engine_version: utils::to_vk_version(&config.engine_version()),
            p_application_name: application_name.as_ptr(),
            p_engine_name: engine_name.as_ptr(),
            api_version: vk::API_VERSION_1_2,
            ..Default::default()
        };

        let mut enabled_layers_names: Vec<*const c_char> = Vec::new();
        let c_validation_layer_name = CString::new(VALIDATION_LAYER_NAME)?;
        if config.enable_validation() {
            enabled_layers_names.push(c_validation_layer_name.as_ptr());
        }
        let enabled_extension_names: Vec<*const c_char> = Vec::new();
        let instance_create_info = vk::InstanceCreateInfo {
            p_application_info: &application_info,
            enabled_layer_count: enabled_layers_names.len() as u32,
            pp_enabled_layer_names: enabled_layers_names.as_ptr(),
            enabled_extension_count: enabled_extension_names.len() as u32,
            pp_enabled_extension_names: enabled_extension_names.as_ptr(),
            ..Default::default()
        };
        let instance = unsafe {
            lib_entry.create_instance(&instance_create_info, None)?
        };

        Ok(Self {
            instance,
            version,
            lib_entry,
            available_layer_properties,
            available_extension_properties,
        })
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn available_layer_properties(&self) -> &Vec<vk::LayerProperties> {
        &self.available_layer_properties
    }

    pub fn available_extension_properties(&self) -> &Vec<vk::ExtensionProperties> {
        &self.available_extension_properties
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
