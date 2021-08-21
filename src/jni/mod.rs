use std::ffi::c_void;

use jni::objects::{JClass, JObject};
use jni::sys::{jint, JNI_ERR, JNI_VERSION_1_6};
use jni::{JNIEnv, JavaVM};

use super::config::ENGINE_NAME;

mod logger;
mod utils;
mod window;

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn JNI_OnLoad(_vm: *mut JavaVM, _reserved: *mut c_void) -> jint {
    match pretty_env_logger::try_init() {
        Ok(_) => {
            log::trace!(target: ENGINE_NAME, "loading library...");
            log::trace!(target: ENGINE_NAME, "logger initialized");
            JNI_VERSION_1_6
        }
        Err(err) => {
            eprintln!("logger initialization error: {}", err);
            JNI_ERR
        }
    }
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn JNI_OnUnload(_vm: *mut JavaVM, _reserved: *mut c_void) {
    log::trace!(target: ENGINE_NAME, "unloading library...")
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn Java_com_tuguzT_native_Entry_initialize(
    env: JNIEnv,
    _class: JClass,
    config: JObject,
) {
    let config = utils::get_config(env, config);
    match config {
        Ok(config) => log::info!(target: ENGINE_NAME, "{:#?}", config),
        Err(err) => log::error!(target: ENGINE_NAME, "initialization error: {:?}", err),
    }
    todo!("engine initialization");
}
