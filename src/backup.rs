use crate::os::unix::UnixFsMetaData;
use crate::repository::BackupBlockId;
use serde::{Deserialize, Serialize};

/**
 * Object representing a single backup instance
 */
#[derive(Serialize, Deserialize)]
pub struct BackupInstance {
    pub name: String,
    pub time: u64,
    pub entry_list_id: BackupBlockId,
}

/**
 * The list of backup entries for this
 */
#[derive(Serialize, Deserialize)]
pub struct EntryList(pub Vec<BackupEntry>);

impl From<Vec<BackupEntry>> for EntryList {
    fn from(entries: Vec<BackupEntry>) -> Self {
        Self(entries)
    }
}

/**
 * The possible meta data types attached to a backup object
 */
#[derive(Serialize, Deserialize)]
pub enum Meta {
    UnixFsMeta(UnixFsMetaData),
}

/**
 * entry representing a single backup object
 */
#[derive(Serialize, Deserialize)]
pub struct BackupEntry {
    /**
     * The name of the backup object
     */
    pub name: String,
    /**
     * The id of the block in the repository containing the block list for this entry
     */
    pub block_list_id: BackupBlockId,
    /**
     * The meta data attached to the object
     */
    pub meta: Meta,
}
