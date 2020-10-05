use std::fmt::Debug;
use std::fmt::Display;

/**
 * Generic error used throughout the program
 */
pub struct Error {
    pub message: &'static str,
    pub cause: Option<std::boxed::Box<dyn std::error::Error>>,
}

/**
 * result type used throughout the program
 */
pub type Result<T> = std::result::Result<T, Error>;

pub fn backrub_error<T>(
    message: &'static str,
    error: Option<Box<dyn std::error::Error>>,
) -> Result<T> {
    Err(Error {
        message: message,
        cause: error,
    })
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.cause {
            Some(e) => write!(fmt, "{} (caused by: {})", self.message, e),
            None => write!(fmt, "{}", self.message),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.cause {
            Some(e) => write!(fmt, "{} (caused by: {})", self.message, e),
            None => write!(fmt, "{}", self.message),
        }
    }
}
