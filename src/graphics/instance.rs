use std::borrow::Cow;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

use ash::extensions::ext;
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
    debug_utils: Option<DebugUtils>,
    instance_loader: ash::Instance,
    entry_loader: ash::Entry,
}

impl Instance {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        // Get entry loader and Vulkan API version
        let entry_loader = unsafe {
            ash::Entry::new()?
        };
        let version = match entry_loader.try_enumerate_instance_version()? {
            Some(version) => utils::from_vk_version(version),
            None => utils::from_vk_version(vk::API_VERSION_1_0),
        };

        // Get available instance properties
        let available_layer_properties = entry_loader
            .enumerate_instance_layer_properties()?;
        let available_extension_properties = entry_loader
            .enumerate_instance_extension_properties()?;

        // Setup application info for Vulkan API
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

        // Initialize containers for raw pointers of names
        let mut enabled_layers_names = Vec::new();
        let mut enabled_extension_names = Vec::new();

        // Push names' pointers into container if validation was enabled
        let debug_utils_extension_name = ext::DebugUtils::name();
        if ENABLE_VALIDATION {
            enabled_layers_names.push(VALIDATION_LAYER_NAME);
            let available_extension_names: Vec<&CStr> =
                available_extension_properties.iter()
                    .map(|extension| unsafe {
                        CStr::from_ptr(extension.extension_name.as_ptr())
                    }).collect();
            if available_extension_names.contains(&debug_utils_extension_name) {
                enabled_extension_names.push(debug_utils_extension_name.as_ptr());
            }
        }

        // Initialize instance create info and get an instance
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

        // Initialize debug utils extension
        let debug_utils = if ENABLE_VALIDATION &&
            enabled_extension_names.contains(&debug_utils_extension_name.as_ptr()) {
            let returnable = DebugUtils::new(&entry_loader, &instance_loader)?;
            log::info!("Vulkan validation layer enabled");
            Some(returnable)
        } else {
            None
        };

        Ok(Self {
            instance_loader,
            version,
            entry_loader,
            available_layer_properties,
            available_extension_properties,
            debug_utils,
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
        self.debug_utils = None;
        unsafe {
            self.instance_loader.destroy_instance(None);
        }
    }
}

struct DebugUtils {
    loader: ext::DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugUtils {
    pub fn new(
        entry_loader: &ash::Entry,
        instance_loader: &ash::Instance,
    ) -> Result<Self, Box<dyn Error>> {
        let messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::all(),
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::all(),
            pfn_user_callback: Some(vulkan_debug_callback),
            ..Default::default()
        };
        let loader = ext::DebugUtils::new(entry_loader, instance_loader);
        let messenger = unsafe {
            loader.create_debug_utils_messenger(&messenger_create_info, None)?
        };
        Ok(Self {
            loader,
            messenger,
        })
    }
}

impl Drop for DebugUtils {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_debug_utils_messenger(self.messenger, None)
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
        "{:?}:{:?} [{} ({})] : {}",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );
    if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE {
        log::trace!("{}", formatted);
    } else if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        log::info!("{}", formatted);
    } else if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        log::warn!("{}", formatted);
    } else {
        log::error!("{}", formatted);
    }
    vk::FALSE
}
