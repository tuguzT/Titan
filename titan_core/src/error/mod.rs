//! Utilities for game engine error handling.

use std::error::Error as StdError;
use std::fmt;

/// Result of any operation which can return an error.
pub type Result<T> = std::result::Result<T, Error>;

/// General error type of game engine.
///
/// Contains general message and source of error, if any.
///
#[derive(Debug)]
pub struct Error {
    message: String,
    source: Option<Box<dyn StdError + Send + Sync + 'static>>,
}

impl Error {
    /// Creates new error with specified message and source of error.
    pub fn new<T, E>(message: T, source: E) -> Self
    where
        T: ToString,
        E: StdError + Send + Sync + 'static,
    {
        Self {
            message: message.to_string(),
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(source) = &self.source {
            write!(f, " ({})", source)
        } else {
            Ok(())
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|err| err.as_ref() as _)
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::from(message.to_string())
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self {
            message,
            source: None,
        }
    }
}
