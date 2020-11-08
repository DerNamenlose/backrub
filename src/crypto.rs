use super::errors::backrub_error;
use super::errors::Result;
use crate::errors::Error;
use aes_gcm_siv::aead::generic_array::GenericArray;
use aes_gcm_siv::aead::{Aead, NewAead};
use aes_gcm_siv::Aes256GcmSiv;
use rand::RngCore;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;

#[derive(Serialize, Deserialize)]
pub struct CryptoBlock {
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub nonce: Vec<u8>,
}

/**
 * Type alias for the keyset used in the repository interface
 */
pub type KeySet = HashMap<u64, DataEncryptionKey>;

#[derive(Serialize, Deserialize)]
pub struct KeyedCryptoBlock {
    pub key_index: u64,
    pub block: CryptoBlock,
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
    pub fn encrypt_block(&self, block: &Vec<u8>) -> Result<CryptoBlock> {
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
    pub fn decrypt_block(&self, block: &CryptoBlock) -> Result<Vec<u8>> {
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

pub fn decode_block<R>(block: R, cipher: &Cipher) -> Result<Vec<u8>>
where
    R: Read,
{
    let mut deserializer = Deserializer::new(block);
    let crypto_block: CryptoBlock = Deserialize::deserialize(&mut deserializer)
        .or_else(|e| backrub_error("Could not deserialize crypto block", Some(e.into())))?;
    cipher.decrypt_block(&crypto_block)
}

pub fn decode_keyed_block<R>(block: R, keys: &KeySet) -> Result<Vec<u8>>
where
    R: Read,
{
    let mut deserializer = Deserializer::new(block);
    let keyed_block: KeyedCryptoBlock = Deserialize::deserialize(&mut deserializer)
        .or_else(|e| backrub_error("Could not deserialize keyed block", Some(e.into())))?;
    let key = keys.get(&keyed_block.key_index).ok_or(Error {
        message: "Key not found",
        cause: None,
    })?;
    let cipher = Cipher::new(&key);
    cipher.decrypt_block(&keyed_block.block)
}

pub fn encode_block<W>(target: W, block: Vec<u8>, cipher: &Cipher) -> Result<()>
where
    W: Write,
{
    let crypto_block = cipher.encrypt_block(&block)?;
    crypto_block
        .serialize(&mut Serializer::new(target))
        .or_else(|e| backrub_error("Could not serialize crypto block", Some(e.into())))?;
    Ok(())
}

pub fn encode_keyed_block<W>(
    target: W,
    block: &Vec<u8>,
    key: &(u64, DataEncryptionKey),
) -> Result<()>
where
    W: Write,
{
    let cipher = Cipher::new(&key.1);
    let crypto_block = cipher.encrypt_block(block)?;
    let keyed_block = KeyedCryptoBlock {
        key_index: key.0,
        block: crypto_block,
    };
    keyed_block
        .serialize(&mut Serializer::new(target))
        .or_else(|e| backrub_error("Could not serialized keyed block", Some(e.into())))?;
    Ok(())
}
