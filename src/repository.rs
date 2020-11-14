use super::backup::BackupInstance;
use super::backupobject::BackupObjectReader;
use super::errors::Result;
use crate::backup::EntryList;
use crate::backupobject::BackupObject;
use crate::crypto::DataEncryptionKey;
use crate::crypto::InputKey;
use crate::crypto::KeySet;
use crate::errors::backrub_error;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
/**
 * Meta information of a repository
 */
#[derive(Serialize, Deserialize)]
pub struct BackrubRepositoryMeta {
    pub version: u32,
    pub title: String,
    pub salt: Vec<u8>,
    pub iterations: u16,
}

/**
 * ID type used for block identifiers
 */
#[derive(Serialize, Deserialize, Eq)]
pub struct BackupBlockId(#[serde(with = "serde_bytes")] Vec<u8>);

impl BackupBlockId {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            backrub_error("Block IDs MUST be 256 bit", None)
        } else {
            let id = Vec::from(bytes);
            Ok(Self(id))
        }
    }
    pub fn to_str(&self) -> String {
        hex::encode(&self.0)
    }
}

impl Display for BackupBlockId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_fmt(format_args!("block({})", hex::encode(&self.0)))
    }
}

impl PartialEq for BackupBlockId {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

pub trait Repository {
    /**
     * Return the meta information of this repository
     */
    fn meta(&self) -> Result<&BackrubRepositoryMeta>;
    /**
     * Initialize the given backup repository, i.e. check whether anything needs to be set up in the
     * directory
     */
    fn initialize(&self, input_key: InputKey) -> Result<()>;
    /**
     * open this repository
     */
    fn open(&mut self, input_key: InputKey) -> Result<()>;
    /**
     * get the keys loaded in this repository
     */
    fn keys(&self) -> Result<&KeySet>;
    /**
     * Get the currently valid data encryption key for the repository.
     * This will be the most recently generated key.
     */
    fn current_key(&self) -> Result<&(u64, DataEncryptionKey)>;
    /**
     * Add a new block to the block store. This will return the
     * block's ID, if successful or an error description, if not
     */
    fn add_block(&self, data: &[u8]) -> Result<(BackupBlockId, usize)>;

    /**
     * Store the list of entries in a backup instance in the block store
     */
    fn store_entry_list(&self, entries: &EntryList) -> Result<(BackupBlockId, usize)>;

    /**
     * load the list of entries in a backup instance from the block store
     */
    fn load_entry_list(&self, list_id: &BackupBlockId) -> Result<EntryList>;

    /**
     * Finish the given backup by writing it to the repository
     */
    fn finish_backup(&self, backup: BackupInstance) -> Result<()>;

    /**
     * Open an object based on its ID
     */
    fn open_object(&self, id: &BackupBlockId) -> Result<BackupObject>;

    /**
     * Open an object reader for an object in the repository
     */
    fn open_object_reader(&self, object: BackupObject) -> Result<Box<dyn BackupObjectReader>>;

    /**
     * List the currently stored backup instances
     */
    fn list_instances(&self) -> Result<Vec<BackupInstance>>;

    /**
     * Load a instance with the given name
     */
    fn open_instance(&self, name: &str) -> Result<BackupInstance>;
}
