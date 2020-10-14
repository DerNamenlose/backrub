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

/**
 * Key type used for actually encrypting data. This is a more temporary
 * type of key, that may be subject to a key schedule (as opposed to the
 * master key or the user input key, which are permanent)
 */
#[derive(Clone)]
pub struct DataEncryptionKey {
    pub created_at: u64,
    pub value: Vec<u8>,
}

impl Cipher {
    pub fn new(key: &DataEncryptionKey) -> Self {
        Cipher {
            cipher: Aes256GcmSiv::new(GenericArray::from_slice(&key.value)),
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
        self.cipher
            .decrypt(GenericArray::from_slice(&block.nonce), block.data.as_ref())
            .or_else(|_| backrub_error("Could not decrypt block.", None))
    }
}

/**
 * Type used for input key, i.e. passwords etc.
 *
 * This is to clearly distinguish between user input keys, master keys (used for key encryption)
 * and data encryption keys (used for actually encrypting data)
 */
pub struct InputKey(Vec<u8>);

impl From<&[u8]> for InputKey {
    fn from(key_data: &[u8]) -> Self {
        Self(Vec::from(key_data))
    }
}

/**
 * Type for a master key as derived from a user input key
 */
pub struct MasterKey(Vec<u8>);

impl From<&MasterKey> for DataEncryptionKey {
    /**
     * explicitly use the master key as a data encryption key (e.g. for key encryption)
     */
    fn from(master_key: &MasterKey) -> Self {
        DataEncryptionKey {
            value: master_key.0.clone(),
            created_at: 0,
        }
    }
}

pub fn derive_key(key: &InputKey, salt: &[u8], iterations: u32) -> Result<MasterKey> {
    let mut config = argon2::Config::default();
    config.variant = argon2::Variant::Argon2id;
    config.time_cost = iterations;
    argon2::hash_raw(&key.0, &salt, &config)
        .map(|key| MasterKey(key))
        .or_else(|e| backrub_error("Could not derive master key", Some(e.into())))
}
