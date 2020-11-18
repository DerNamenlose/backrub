use crate::common::read_key;
use crate::errors::Result;
use crate::fsrepository::FsRepository;
use crate::repository::Repository;
use std::path::Path;

pub fn show(repository: &Path, name: &String, contents: bool) -> Result<()> {
    let mut repo = FsRepository::new(&repository);
    let key = read_key()?;
    repo.open(key)?;
    let instance = repo.open_instance(&name)?;
    println!("-----\n{}\n-----", instance);
    if contents {
        let entries = repo.load_entry_list(&instance.entry_list_id)?;
        for entry in entries.0 {
            println!("({}) {}", entry.entry_type, entry.name);
        }
    }
    Ok(())
}
