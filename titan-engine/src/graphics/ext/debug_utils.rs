use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_void;

use ash::extensions::ext::DebugUtils as DebugUtilsLoader;
use ash::vk;
use log::Level;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::super::{
    instance::{self, Instance},
    slotmap::SlotMappable,
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct DebugUtils {
    key: Key,
    loader: DebugUtilsLoader,
    messenger: vk::DebugUtilsMessengerEXT,
    parent_instance: instance::Key,
}

impl DebugUtils {
    pub fn new(instance_key: instance::Key) -> Result<Key> {
        let slotmap = SlotMappable::slotmap().read().unwrap();
        let instance: &Instance = slotmap.get(instance_key).expect("instance not found");

        let messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .pfn_user_callback(Some(self::callback));
        let loader = DebugUtilsLoader::new(instance.entry_loader(), instance.loader());
        let messenger =
            unsafe { loader.create_debug_utils_messenger(&messenger_create_info, None)? };

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            loader,
            messenger,
            parent_instance: instance_key,
        });
        Ok(key)
    }

    pub fn parent_instance(&self) -> instance::Key {
        self.parent_instance
    }

    pub fn name() -> &'static CStr {
        DebugUtilsLoader::name()
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
    if p_callback_data.is_null() {
        return vk::FALSE;
    }

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
        target: "titan_engine::graphics::log",
        level,
        "{:?} [{} ({})] : {}",
        message_type,
        message_id_name,
        message_id_number,
        message,
    );
    vk::FALSE
}
