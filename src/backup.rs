use crate::os::unix::UnixFsMeta;
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
    UnixMeta(UnixFsMeta),
}

impl Display for Meta {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        match &self {
            Meta::UnixMeta(meta_data) => write!(formatter, "{}", meta_data),
        }
    }
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
     * The type-specific information for this entry
     */
    pub entry_type: EntryType,
    /**
     * The meta data attached to the object
     */
    pub meta: Meta,
}

/**
 * The entry type of the backup object
 */
#[derive(Serialize, Deserialize)]
pub enum EntryType {
    /**
     * The entry is a file with the given data
     */
    File(FileEntryData),
    /**
     * The entry is a directory
     */
    Dir,
    /**
     * The file is a (sym)link with the gven link data
     */
    Link(LinkData),
}

/**
 * entry representing a single file object
 */
#[derive(Serialize, Deserialize)]
pub struct FileEntryData {
    /**
     * The id of the block in the repository containing the block list for this entry
     */
    pub block_list_id: BackupBlockId,
}

/**
 * Data associated with a symlink
 */
#[derive(Serialize, Deserialize)]
pub struct LinkData {
    /**
     * The target path the link points to
     */
    pub target: String,
}
