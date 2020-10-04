use super::backup::BackupInstance;
use super::backupobject::BackupObjectWriter;
use crate::backupobject::BackupObjectReader;
use std::io;

pub trait Repository {
    /**
     * reate a new repository instance pointing to the given path
     */
    fn new(path: &str) -> Self;
    /**
     * Initialize the given backup repository, i.e. check whether anything needs to be set up in the
     * directory
     */
    fn initialize(&self) -> io::Result<()>;

    /**
     * Start a new backup object in the repository
     */
    fn start_object(&self, name: &str) -> std::result::Result<Box<dyn BackupObjectWriter>, String>;

    /**
     * Finish the given backup by writing it to the repository
     */
    fn finish_backup(&self, backup: BackupInstance) -> std::io::Result<()>;

    /**
     * Open an object in the repository
     */
    fn open_object(&self, id: &str) -> std::result::Result<Box<dyn BackupObjectReader>, String>;
}
