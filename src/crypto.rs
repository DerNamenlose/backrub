use super::errors::backrub_error;
use super::errors::Result;
use aes_gcm_siv::aead::generic_array::GenericArray;
use aes_gcm_siv::aead::{Aead, NewAead};
use aes_gcm_siv::Aes256GcmSiv;
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CryptoBlock {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub struct Cipher {
    cipher: Aes256GcmSiv,
}

impl Cipher {
    pub fn new(key: Vec<u8>) -> Self {
        Cipher {
            cipher: Aes256GcmSiv::new(GenericArray::from_slice(&key)),
        }
    }
    pub fn encrypt_block(&self, block: Vec<u8>) -> Result<CryptoBlock> {
        let mut nonce = [0; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let encrypted = self
            .cipher
            .encrypt(GenericArray::from_slice(&nonce), block.as_ref())
            .or_else(|_e| backrub_error("Encrypting block failed", None))?;
        Ok(CryptoBlock {
            data: encrypted,
            nonce: Vec::from(nonce),
        })
    }
    pub fn decrypt_block(&self, block: CryptoBlock) -> Result<Vec<u8>> {
        let decrypted = self
            .cipher
            .decrypt(GenericArray::from_slice(&block.nonce), block.data.as_ref())
            .or_else(|e| backrub_error("Decrypting block failed", None))?;
        return Ok(decrypted);
    }
}

pub fn derive_key(key: &[u8], salt: &[u8], iterations: u32) -> Vec<u8> {
    let mut config = argon2::Config::default();
    config.variant = argon2::Variant::Argon2id;
    config.time_cost = iterations;
    argon2::hash_raw(key, &salt, &config).unwrap()
}
