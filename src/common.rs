use super::crypto::InputKey;
use super::errors::{backrub_error, Result};
use serde::{Deserialize, Serialize};

pub fn read_key() -> Result<InputKey> {
    let key = std::env::var("BACKRUB_KEY")
        .or_else(|_| rpassword::prompt_password_stdout("Repository password: "))
        .or_else(|e| backrub_error("Could not read password.", Some(e.into())))?;
    Ok(InputKey::from(key.as_bytes()))
}

/**
 * internal representation of the the CryptoBlock struct to enable more efficient serialization
 */
#[derive(Serialize, Deserialize)]
pub struct InternalCryptoBlock {
    pub data: serde_bytes::ByteBuf,
    pub nonce: serde_bytes::ByteBuf,
}
