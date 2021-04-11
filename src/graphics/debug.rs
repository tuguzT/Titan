use std::borrow::Cow;
use std::error::Error;
use std::ffi::CStr;
use std::os::raw::c_void;

use ash::extensions::ext;
use ash::vk;

pub struct DebugUtils {
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
            pfn_user_callback: Some(callback),
            ..Default::default()
        };
        let loader = ext::DebugUtils::new(entry_loader, instance_loader);
        let messenger =
            unsafe { loader.create_debug_utils_messenger(&messenger_create_info, None)? };
        Ok(Self { loader, messenger })
    }

    pub fn name() -> &'static CStr {
        ext::DebugUtils::name()
    }
}

impl Drop for DebugUtils {
    fn drop(&mut self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None)
        }
    }
}

unsafe extern "system" fn callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    if !p_callback_data.is_null() {
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
            message_severity, message_type, message_id_name, message_id_number, message,
        );
        match message_severity {
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
                log::trace!("{}", formatted)
            }
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
                log::info!("{}", formatted)
            }
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
                log::warn!("{}", formatted)
            }
            _ => {
                log::error!("{}", formatted)
            }
        }
    }
    vk::FALSE
}
