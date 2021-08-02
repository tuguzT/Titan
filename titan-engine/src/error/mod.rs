use std::error::Error as StdError;
use std::fmt;

use ash::vk;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Graphics {
        result: vk::Result,
    },
    Other {
        message: String,
        source: Option<Box<dyn StdError>>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Graphics { result } => write!(f, "{:?}: {}", result, result),
            Self::Other { message, .. } => write!(f, "{}", message),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Graphics { result } => Some(result),
            Self::Other { source, .. } => source.as_ref().map(Box::as_ref),
        }
    }
}

impl From<vk::Result> for Error {
    fn from(result: vk::Result) -> Self {
        Self::Graphics { result }
    }
}
