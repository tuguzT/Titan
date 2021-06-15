use std::borrow::Cow;
use std::error::Error;
use std::ffi::CStr;
use std::os::raw::c_void;

use ash::extensions::ext::DebugUtils as AshDebugUtils;
use ash::vk;
use log::Level;

use crate::graphics::slotmap::{InstanceKey, SLOTMAP_INSTANCE};
use crate::graphics::utils;

pub struct DebugUtils {
    loader: AshDebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
    parent_instance: InstanceKey,
}

impl DebugUtils {
    pub fn new(instance_key: InstanceKey) -> Result<Self, Box<dyn Error>> {
        let slotmap = SLOTMAP_INSTANCE.read()?;
        let instance = slotmap
            .get(instance_key)
            .ok_or_else(|| utils::make_error("instance not found"))?;

        let messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .pfn_user_callback(Some(self::callback));
        let loader = AshDebugUtils::new(instance.entry_loader(), instance.loader());
        let messenger =
            unsafe { loader.create_debug_utils_messenger(&messenger_create_info, None)? };

        Ok(Self {
            loader,
            messenger,
            parent_instance: instance_key,
        })
    }

    pub fn parent_instance(&self) -> InstanceKey {
        self.parent_instance
    }

    pub fn name() -> &'static CStr {
        AshDebugUtils::name()
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

        let level = match message_severity {
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => Level::Trace,
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO => Level::Info,
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => Level::Warn,
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => Level::Error,
            _ => unreachable!(),
        };
        log::log!(
            target: "titan_engine::graphics::debug",
            level,
            "{:?}:{:?} [{} ({})] : {}",
            message_severity,
            message_type,
            message_id_name,
            message_id_number,
            message,
        );
    }
    vk::FALSE
}
