use std::ffi::c_void;
use std::result::Result::Ok;

use jni::{JavaVM, JNIEnv};
use jni::objects::{JClass, JObject};
use jni::sys::{jint, JNI_ERR, JNI_VERSION_1_6};

use crate::jni::utils::get_config;

mod logger;
mod window;
mod utils;

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn JNI_OnLoad(_vm: *mut JavaVM, _reserved: *mut c_void) -> jint {
    match pretty_env_logger::try_init() {
        Ok(_) => {
            log::trace!(target: "titan-rs", "Loading library...");
            log::trace!(target: "titan-rs", "Logger initialized");
            JNI_VERSION_1_6
        },
        Err(err) => {
            eprintln!("Logger initialization error: {}", err);
            JNI_ERR
        }
    }
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn JNI_OnUnload(_vm: *mut JavaVM, _reserved: *mut c_void) {
    log::trace!(target: "titan-rs", "Unloading library...")
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn Java_com_tuguzT_native_Entry_initialize(env: JNIEnv, _class: JClass, config: JObject) {
    let config = get_config(env, config);
    match config {
        Ok(config) => log::info!(target: "titan-rs", "{:#?}", config),
        Err(err) => log::error!(target: "titan-rs", "Initialization error: {:?}", err)
    }
}
