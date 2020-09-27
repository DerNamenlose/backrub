use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BackupInstance {
    pub name: String,
    pub time: u64,
    pub entries: Vec<String>,
    pub key: String,
}
