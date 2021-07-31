use std::ffi::CStr;
use std::ops::Deref;
use std::os::raw::c_void;

use ash::extensions::ext::DebugUtils as DebugUtilsLoader;
use ash::vk;
use log::Level;

use proc_macro::SlotMappable;

use crate::error::Result;

use super::super::{
    instance::{self, Instance},
    slotmap::{HasParent, SlotMappable},
    utils::{HasHandle, HasLoader},
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct DebugUtils {
    #[key]
    key: Key,
    loader: DebugUtilsLoader,
    messenger: vk::DebugUtilsMessengerEXT,
    parent_instance: instance::Key,
}

impl HasParent<Instance> for DebugUtils {
    fn parent_key(&self) -> instance::Key {
        self.parent_instance
    }
}

impl HasLoader for DebugUtils {
    type Loader = DebugUtilsLoader;

    fn loader(&self) -> Box<dyn Deref<Target = Self::Loader> + '_> {
        Box::new(&self.loader)
    }
}

impl HasHandle for DebugUtils {
    type Handle = vk::DebugUtilsMessengerEXT;

    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_> {
        Box::new(&self.messenger)
    }
}

impl DebugUtils {
    pub fn new(instance_key: instance::Key) -> Result<Key> {
        let slotmap = SlotMappable::slotmap().read().unwrap();
        let instance: &Instance = slotmap.get(instance_key).expect("instance not found");

        let loader = instance.loader();
        let loader = DebugUtilsLoader::new(loader.entry(), loader.instance());

        let messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .pfn_user_callback(Some(self::callback));
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
    let callback_data = match p_callback_data.as_ref() {
        None => return vk::FALSE,
        Some(data) => data,
    };

    let message_id_number = callback_data.message_id_number;
    let message_id_name = callback_data
        .p_message_id_name
        .as_ref()
        .map_or("None", |ptr| CStr::from_ptr(ptr).to_str().unwrap());
    let message = callback_data
        .p_message
        .as_ref()
        .map_or("None", |ptr| CStr::from_ptr(ptr).to_str().unwrap());

    let level = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => Level::Trace,
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => Level::Info,
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => Level::Warn,
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => Level::Error,
        _ => unreachable!(),
    };
    log::log!(
        target: "titan_engine::graphics::debug_utils",
        level,
        "{:?} [{} ({})] : {}",
        message_type,
        message_id_name,
        message_id_number,
        message,
    );
    vk::FALSE
}
