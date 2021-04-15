use std::fmt::{Debug, Display, Formatter, Result};

pub struct Error {
    pub message: String,
    pub r#type: ErrorType,
}

impl Error {
    pub fn new(message: String, r#type: ErrorType) -> Self {
        Self { message, r#type }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "TITAN {:?} error: {}", self.r#type, self.message)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "TITAN {:?}: {}", self.r#type, self.message)
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum ErrorType {
    Graphics,
}
