use super::common::read_key;
use super::errors::backrub_error;
use super::errors::Result;
use super::fsrepository::FsRepository;
use super::repository::Repository;
use crate::backup::BackupEntry;
use crate::crypto::decode_keyed_block;
use crate::os::unix::set_meta_data;
use std::io::Cursor;
use std::io::Write;

pub fn restore_backup(repository: &str, path: &str, name: &str) -> Result<()> {
    log::info!(
        "Restoring {} from repository {} to {}",
        name,
        repository,
        path
    );
    let mut repository: FsRepository = Repository::new(repository);
    let key = read_key()?;
    repository.open(key)?;
    let instance = repository.open_instance(name)?;
    let entries = repository.load_entry_list(&instance.entry_list_id)?;
    let mut errors = vec![];
    for entry in entries.0 {
        let restore_result = restore_entry(&repository, &entry, path);
        match restore_result {
            Ok(_) => log::debug!("Successfully restored object"),
            Err(e) => errors.push(e),
        }
    }
    if errors.len() != 0 {
        log::error!("{} error(s) occured during restore", errors.len());
        for error in errors {
            log::error!("{}", &error);
            println!("{}", &error);
        }
        backrub_error("Restore unsuccessful", None)
    } else {
        Ok(())
    }
}

fn restore_entry(repo: &FsRepository, entry: &BackupEntry, base_path: &str) -> Result<()> {
    let restore_path: std::path::PathBuf = [base_path, &entry.name].iter().collect();
    let parent_path = restore_path.parent().ok_or(super::errors::Error {
        message: "Object has no parent directory",
        cause: None,
    })?;
    log::debug!(
        "Restoring {} to {}",
        &entry.name,
        restore_path.as_path().to_str().unwrap()
    );
    std::fs::create_dir_all(parent_path)
        .or_else(|e| backrub_error("Could not create parent path", Some(e.into())))?;
    let mut file = std::fs::File::create(&restore_path)
        .or_else(|e| backrub_error("Could not create output file", Some(e.into())))?;
    let object = repo.open_object(&entry.block_list_id)?;
    let object_reader = repo.open_object_reader(object)?;
    let keyset = repo.keys()?;
    for block in object_reader.blocks() {
        log::debug!("Decoding serialized data block of size {}", block.len());
        let data_block = decode_keyed_block(Cursor::new(block), &keyset)?;
        log::debug!("Contained block of size {}", data_block.len());
        file.write(&data_block)
            .or_else(|e| backrub_error("Could not write to output file", Some(e.into())))?;
    }
    set_meta_data(&restore_path, &entry.meta)?;
    Ok(())
}
