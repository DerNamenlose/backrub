use serde::{Deserialize, Serialize};

/**
 * A writer for writing backup objects to a repository.
 * This trait is implemented by different repository types to
 * implement their version of the writer
 */
pub trait BackupObjectWriter {
    /**
     * Add a new block to the backup object. This will return the
     * block's ID, if successful or an error description, if not
     */
    fn add_block(&mut self, data: &[u8]) -> Result<String, &'static str>;

    /**
     * Finish this object by writing
     */
    fn finish(&self) -> Result<String, &'static str>;
}

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
    pub name: String,
    pub blocks: Vec<String>,
}
