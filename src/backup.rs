use crate::os::unix::UnixFsMetaData;
use crate::repository::BackupBlockId;
use chrono::DateTime;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::Add;

/**
 * Object representing a single backup instance
 */
#[derive(Serialize, Deserialize)]
pub struct BackupInstance {
    pub name: String,
    pub time: u64,
    pub entry_list_id: BackupBlockId,
}

impl Display for BackupInstance {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let date: DateTime<Local> = std::time::SystemTime::UNIX_EPOCH
            .add(std::time::Duration::from_secs(self.time))
            .into();
        write!(fmt, "Name: {}\ncreated at: {}", &self.name, date)
    }
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
