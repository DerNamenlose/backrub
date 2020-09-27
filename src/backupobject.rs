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

#[derive(Deserialize, Serialize)]
pub struct BackupObject {
    pub name: String,
    pub blocks: Vec<String>,
}
