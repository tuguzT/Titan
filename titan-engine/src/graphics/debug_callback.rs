use log::Level;
use vulkano::instance::debug::{Message, MessageSeverity, MessageType};

pub fn callback(message: &Message) {
    let level = match message.severity {
        MessageSeverity { verbose, .. } if verbose => Level::Trace,
        MessageSeverity { information, .. } if information => Level::Info,
        MessageSeverity { warning, .. } if warning => Level::Warn,
        MessageSeverity { error, .. } if error => Level::Error,
        _ => Level::Debug,
    };
    let ty = match message.ty {
        MessageType { general, .. } if general => "GENERAL",
        MessageType { validation, .. } if validation => "VALIDATION",
        MessageType { performance, .. } if performance => "PERFORMANCE",
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
