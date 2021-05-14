use std::ffi::c_void;

use jni::JavaVM;
use jni::sys::{jint, JNI_ERR, JNI_VERSION_1_6};

mod logger;

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
