use std::ffi::c_void;

use jni::{JavaVM, JNIEnv};
use jni::objects::{JClass, JString};
use jni::sys::{jint, JNI_ERR, JNI_VERSION_1_6};
use log::Level;

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

fn handle_log(env: JNIEnv, message: JString, level: Level) {
    let message = env.get_string(message);
    match message {
        Ok(message) => {
            let message: String = message.into();
            log::log!(target: "titan-rs", level, "{}", message);
        }
        Err(err) => log::error!(target: "titan-rs", "{:?}", err)
    }
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn Java_native_Native_error(env: JNIEnv, _class: JClass, message: JString) {
    handle_log(env, message, Level::Error)
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn Java_native_Native_warn(env: JNIEnv, _class: JClass, message: JString) {
    handle_log(env, message, Level::Warn)
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn Java_native_Native_info(env: JNIEnv, _class: JClass, message: JString) {
    handle_log(env, message, Level::Info)
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn Java_native_Native_debug(env: JNIEnv, _class: JClass, message: JString) {
    handle_log(env, message, Level::Debug)
}
