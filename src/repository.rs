use super::backup::BackupInstance;
use super::backupobject::BackupObjectReader;
use super::backupobject::BackupObjectWriter;
use super::errors::Result;

pub trait Repository {
    /**
     * reate a new repository instance pointing to the given path
     */
    fn new(path: &str) -> Self;
    /**
     * Initialize the given backup repository, i.e. check whether anything needs to be set up in the
     * directory
     */
    fn initialize(&self) -> Result<()>;

    /**
     * Start a new backup object in the repository
     */
    fn start_object(&self, name: &str) -> Result<Box<dyn BackupObjectWriter>>;

    /**
     * Finish the given backup by writing it to the repository
     */
    fn finish_backup(&self, backup: BackupInstance) -> Result<()>;

    /**
     * Open an object in the repository
     */
    fn open_object(&self, id: &str) -> Result<Box<dyn BackupObjectReader>>;

    // fn open_instance(&self, name: &str) fdsdff;
}
