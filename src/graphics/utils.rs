#[macro_export]
macro_rules! c_str {
    ($s:expr) => {
        unsafe {
            std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr() as *const std::os::raw::c_char)
        }
    };
}
