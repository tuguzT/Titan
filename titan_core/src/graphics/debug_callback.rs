//! Graphics debugging utilities for game engine.

use std::sync::Arc;

use log::Level;
use vulkano::instance::debug::{
    DebugCallback, DebugCallbackCreationError, Message, MessageSeverity, MessageType,
};
use vulkano::instance::Instance;

/// Create debug callback for validation via Vulkan SDK.
///
/// Note that Khronos validation layer must be enabled.
///
pub fn create_debug_callback(
    instance: &Arc<Instance>,
    severity: MessageSeverity,
    ty: MessageType,
) -> Result<DebugCallback, DebugCallbackCreationError> {
    DebugCallback::new(instance, severity, ty, self::user_callback)
}

/// The actual callback validation function.
///
/// Logs message into global logger.
///
#[rustfmt::skip]
fn user_callback(message: &Message) {
    let level = match message.severity {
        MessageSeverity { verbose: true, .. } => Level::Trace,
        MessageSeverity { information: true, .. } => Level::Info,
        MessageSeverity { warning: true, .. } => Level::Warn,
        MessageSeverity { error: true, .. } => Level::Error,
        _ => Level::Trace,
    };
    let ty = match message.ty {
        MessageType { general: true, .. } => "GENERAL",
        MessageType { validation: true, .. } => "VALIDATION",
        MessageType { performance: true, .. } => "PERFORMANCE",
        _ => "UNKNOWN",
    };
    let layer_prefix = message.layer_prefix.unwrap_or("Unknown");
    let description = message.description;

    log::log!(
        level,
        r#"{} [layer "{}"]: "{}""#,
        ty,
        layer_prefix,
        description,
    );
}
