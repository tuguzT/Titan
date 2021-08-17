#[rustfmt:skip]

use log::Level;
use vulkano::instance::debug::{Message, MessageSeverity, MessageType};

pub fn callback(message: &Message) {
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
        _ => "NONE",
    };
    log::log!(
        level,
        r#"{} [layer "{}"]: "{}""#,
        ty,
        message.layer_prefix.unwrap_or("None"),
        message.description,
    );
}
