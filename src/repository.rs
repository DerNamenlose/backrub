use super::backup::BackupInstance;
use super::backupobject::BackupObjectReader;
use super::backupobject::BackupObjectWriter;
use super::errors::Result;
use crate::backupobject::BackupObject;
use serde::{Deserialize, Serialize};
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

pub trait Repository {
    /**
     * reate a new repository instance pointing to the given path
     */
    fn new(path: &str) -> Self;
    /**
     * Return the meta information of this repository
     */
    fn meta(&self) -> Result<&BackrubRepositoryMeta>;
    /**
     * Initialize the given backup repository, i.e. check whether anything needs to be set up in the
     * directory
     */
    fn initialize(&self) -> Result<()>;
    /**
     * open this repository
     */
    fn open(&mut self) -> Result<()>;
    /**
     * Start a new backup object in the repository
     */
    fn start_object(&self, name: &str) -> Result<Box<dyn BackupObjectWriter>>;

    /**
     * Finish the given backup by writing it to the repository
     */
    fn finish_backup(&self, backup: BackupInstance) -> Result<()>;

    /**
     * Open an object based on its ID
     */
    fn open_object(&self, id: &str) -> Result<BackupObject>;

    /**
     * Open an object reader for an object in the repository
     */
    fn open_object_reader(&self, object: BackupObject) -> Result<Box<dyn BackupObjectReader>>;

    /**
     * List the currently stored backup instances
     */
    fn list_instances(&self) -> Result<Vec<String>>;

    /**
     * Load a instance with the given name
     */
    fn open_instance(&self, name: &str) -> Result<BackupInstance>;
}
