use crate::common::read_key;
use crate::errors::Result;
use crate::fsrepository::FsRepository;
use crate::repository::Repository;
use std::path::Path;

pub fn instances(repository: &Path) -> Result<()> {
    let mut repo = FsRepository::new(&repository);
    let master_key = read_key()?;
    repo.open(master_key)?;
    println!("Opening backup instances...\n");
    for instance in repo.list_instances()? {
        println!("{}\n-----", instance);
    }
    Ok(())
}
