//! API for simple game engine based on Rust and Vulkan API.

pub use app::init;

pub mod app;
pub mod config;
pub mod ecs;
pub mod error;
pub mod window;

mod graphics;
#[cfg(feature = "jni-export")]
mod jni;
