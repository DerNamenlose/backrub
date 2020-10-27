use crate::repository::BackupBlockId;
use serde::{Deserialize, Serialize};

/**
 * interface for getting access to the blocks stored in a backed-up
 * object
 */
pub trait BackupObjectReader {
    /**
     * The iterator representing the data blocks stored in the object
     */
    fn blocks<'a>(&'a self) -> Box<dyn Iterator<Item = Vec<u8>> + 'a>;
}

#[derive(Deserialize, Serialize, Eq, PartialEq)]
pub struct BackupObject {
    pub blocks: Vec<BackupBlockId>,
}
