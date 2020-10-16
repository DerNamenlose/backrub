use serde::{Deserialize, Serialize};

/**
 * Object representing a single backup instance
 */
#[derive(Serialize, Deserialize)]
pub struct BackupInstance {
    pub name: String,
    pub time: u64,
    pub entries: Vec<BackupEntry>,
}

/**
 * entry representing a single backup object
 */
#[derive(Serialize, Deserialize)]
pub struct BackupEntry {
    pub name: String,
    pub block_list_id: String,
}
