use std::borrow::Cow;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

use ash::extensions::ext::DebugUtils;
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;

use crate::config::Config;
use crate::graphics::utils;
use crate::version::Version;

const VALIDATION_LAYER_NAME: *const c_char = crate::c_str_ptr!("VK_LAYER_KHRONOS_validation");

const ENABLE_VALIDATION: bool = cfg!(debug_assertions);

pub struct Instance {
    version: Version,
    available_layer_properties: Vec<vk::LayerProperties>,
    available_extension_properties: Vec<vk::ExtensionProperties>,
    debug_utils_loader: Option<DebugUtils>,
    debug_utils_messenger: Option<vk::DebugUtilsMessengerEXT>,
    instance_loader: ash::Instance,
    entry_loader: ash::Entry,
}

impl Instance {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let entry_loader = unsafe {
            ash::Entry::new()?
        };
        let version = match entry_loader.try_enumerate_instance_version()? {
            Some(version) => utils::from_vk_version(version),
            None => utils::from_vk_version(vk::API_VERSION_1_0),
        };
        let available_layer_properties = entry_loader
            .enumerate_instance_layer_properties()?;
        let available_extension_properties = entry_loader
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
        let mut enabled_extension_names: Vec<*const c_char> = Vec::new();
        if ENABLE_VALIDATION {
            enabled_layers_names.push(VALIDATION_LAYER_NAME);
            let available_extension_names: Vec<&CStr> =
                available_extension_properties.iter()
                    .map(|extension| unsafe {
                        CStr::from_ptr(extension.extension_name.as_ptr())
                    }).collect();
            let p_debug_utils_extension_name = DebugUtils::name();
            if available_extension_names.contains(&p_debug_utils_extension_name) {
                enabled_extension_names.push(p_debug_utils_extension_name.as_ptr());
            }
        }
        let instance_create_info = vk::InstanceCreateInfo {
            p_application_info: &application_info,
            enabled_layer_count: enabled_layers_names.len() as u32,
            pp_enabled_layer_names: enabled_layers_names.as_ptr(),
            enabled_extension_count: enabled_extension_names.len() as u32,
            pp_enabled_extension_names: enabled_extension_names.as_ptr(),
            ..Default::default()
        };
        let instance_loader = unsafe {
            entry_loader.create_instance(&instance_create_info, None)?
        };

        let mut debug_utils_loader = None;
        let mut debug_utils_messenger = None;
        if ENABLE_VALIDATION && enabled_extension_names.contains(&DebugUtils::name().as_ptr()) {
            let debug_utils_messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
                message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                message_type: vk::DebugUtilsMessageTypeFlagsEXT::all(),
                pfn_user_callback: Some(vulkan_debug_callback),
                ..Default::default()
            };
            debug_utils_loader = Some(DebugUtils::new(&entry_loader, &instance_loader));
            debug_utils_messenger = Some(unsafe {
                let reference = debug_utils_loader.as_ref().unwrap();
                reference.create_debug_utils_messenger(
                    &debug_utils_messenger_create_info, None,
                )?
            });
            log::info!("Vulkan validation layer enabled");
        };

        Ok(Self {
            instance_loader,
            version,
            entry_loader,
            available_layer_properties,
            available_extension_properties,
            debug_utils_loader,
            debug_utils_messenger,
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

    pub fn instance_loader(&self) -> &ash::Instance {
        &self.instance_loader
    }

    pub fn entry_loader(&self) -> &ash::Entry {
        &self.entry_loader
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            if let Some(debug_utils_loader) = &self.debug_utils_loader {
                debug_utils_loader.destroy_debug_utils_messenger(
                    self.debug_utils_messenger.unwrap(), None
                );
            }
            self.instance_loader.destroy_instance(None);
        }
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("None")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("None")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    let formatted = format!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );
    if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE {
        log::trace!("{}", formatted)
    } else if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        log::info!("{}", formatted);
    } else if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        log::warn!("{}", formatted);
    } else {
        log::error!("{}", formatted);
    }
    vk::FALSE
}
