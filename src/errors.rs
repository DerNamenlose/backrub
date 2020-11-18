use std::fmt::Debug;
use std::fmt::Display;

/**
 * Generic error used throughout the program
 */
pub struct Error {
    pub message: &'static str,
    pub cause: Option<std::boxed::Box<dyn std::error::Error>>,
    pub is_warning: bool,
}

/**
 * result type used throughout the program
 */
pub type Result<T> = std::result::Result<T, Error>;

pub fn error<T>(message: &'static str, error: Option<Box<dyn std::error::Error>>) -> Result<T> {
    Err(Error {
        message: message,
        cause: error,
        is_warning: false,
    })
}

pub fn warning<T>(message: &'static str, error: Option<Box<dyn std::error::Error>>) -> Result<T> {
    Err(Error {
        message: message,
        cause: error,
        is_warning: true,
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
