use jni::objects::JClass;
use jni::JNIEnv;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_native_Native_hello(_env: JNIEnv, _class: JClass) {
    println!("Hello, World!")
}
