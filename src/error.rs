use std::fmt::{Debug, Display, Formatter, Result};

pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "TITAN error: {}", self.message)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}
