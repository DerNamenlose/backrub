use super::crypto::InputKey;
use super::errors::{backrub_error, Result};

pub fn read_key() -> Result<InputKey> {
    let key = std::env::var("BACKRUB_KEY")
        .or_else(|_| rpassword::prompt_password_stdout("Repository password: "))
        .or_else(|e| backrub_error("Could not read password.", Some(e.into())))?;
    Ok(InputKey::from(key.as_bytes()))
}
