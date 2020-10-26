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
 * entry representing a single backup object
 */
#[derive(Serialize, Deserialize)]
pub struct BackupEntry {
    pub name: String,
    pub block_list_id: BackupBlockId,
}
