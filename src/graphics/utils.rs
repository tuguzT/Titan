#[macro_export]
macro_rules! c_str {
    ($s:expr) => (
        concat!($s, "\0") as *const c_char
    );
}
