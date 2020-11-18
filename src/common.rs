use super::crypto::InputKey;
use super::errors::{error, Result};
use std::fmt::Display;

pub fn read_key() -> Result<InputKey> {
    let key = std::env::var("BACKRUB_KEY")
        .or_else(|_| rpassword::prompt_password_stdout("Repository password: "))
        .or_else(|e| error("Could not read password.", Some(e.into())))?;
    Ok(InputKey::from(key.as_bytes()))
}

pub struct ByteSize(pub usize);

static UNITS: [&'static str; 6] = ["", "kiB", "MiB", "GiB", "TiB", "PiB"];

impl Display for ByteSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut divider = 1;
        for unit in &UNITS {
            if self.0 / divider < 1024 || *unit == "PiB" {
                return write!(f, "{:1} {}", self.0 / divider, unit);
            }
            divider *= 1024;
        }
        Ok(())
    }
}
