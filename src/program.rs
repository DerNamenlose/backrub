use super::backup::BackupInstance;
use super::backupobject::BackupObject;
use super::backupobject::{BackupObjectReader, BackupObjectWriter};
use super::errors::backrub_error;
use super::errors::Result;
use super::fsrepository::FsRepository;
use super::fssource::{FsBlockSource, FsSource};
use super::repository::Repository;
use std::io::Write;
use std::time::SystemTime;

pub fn make_backup(repository: &str, path: &str, name: &str) -> Result<()> {
    let repo: FsRepository = Repository::new(&repository);
    let source: FsSource = FsSource::new(&path);

    repo.initialize()?;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Could not get current time");
    let mut backup_instance = BackupInstance {
        name: String::from(name),
        entries: Vec::new(),
        time: now.as_secs(),
        key: String::new(),
    };
    for file in source.objects() {
        let source_name = file.path().to_str();
        let source_name_relative_result = file.path().strip_prefix(path);
        let source_name_relative = match source_name_relative_result {
            Ok(p) => p.to_str(),
            Err(_) => None,
        };
        if source_name.is_some() && source_name_relative.is_some() {
            let blocks_result = source.open_entry(&source_name.unwrap());
            let object_result = repo.start_object(&source_name_relative.unwrap());
            match (blocks_result, object_result) {
                (Ok(blocks), Ok(mut object)) => {
                    let copy_result = backup_blocks(blocks, object.as_mut());
                    match copy_result {
                        Ok(()) => {
                            println!("Adding object descriptor to repository");
                            let finish_result = object.finish();
                            match finish_result {
                                Ok(id) => {
                                    println!("New object: {}", id);
                                    backup_instance.entries.push(id);
                                }
                                Err(message) => println!(
                                    "Could not finish object {}. Reason: {}",
                                    source_name.unwrap(),
                                    message
                                ),
                            }
                        }
                        Err(message) => println!(
                            "Could not copy blocks for {}. Reason: {}",
                            source_name.unwrap(),
                            message
                        ),
                    }
                }
                (_, _) => println!("Could not copy source blocks into target object"),
            }
        }
    }
    println!("Finishing backup");
    repo.finish_backup(backup_instance)
        .or_else(|e| backrub_error("Could not finish backup instance", Some(e.into())))?;
    println!("Finished backup");
    Ok(())
}

pub fn restore_backup(repository: &str, path: &str, name: &str) -> Result<()> {
    let repository: FsRepository = Repository::new(repository);
    let instance = repository.open_instance(name)?;
    let mut errors = vec![];
    for entry in instance.entries {
        let object_result = repository.open_object(&entry);
        match object_result {
            Ok(object) => {
                let restore_result = restore_object(&repository, object, entry, path);
                match restore_result {
                    Ok(_) => (),
                    Err(e) => errors.push(e),
                }
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }
    Ok(())
}

fn restore_object(
    repo: &FsRepository,
    object: BackupObject,
    entry: String,
    base_path: &str,
) -> Result<()> {
    let restore_path: std::path::PathBuf = [base_path, &object.name].iter().collect();
    let parent_path = restore_path.parent().ok_or(super::errors::Error {
        message: "Object has no parent directory",
        cause: None,
    })?;
    println!(
        "Restoring {} to {}",
        &object.name,
        restore_path.as_path().to_str().unwrap()
    );
    std::fs::create_dir_all(parent_path)
        .or_else(|e| backrub_error("Could not create parent path", Some(e.into())))?;
    let mut file = std::fs::File::create(&restore_path)
        .or_else(|e| backrub_error("Could not create output file", Some(e.into())))?;
    let object_reader = repo.open_object_reader(object)?;
    for block in object_reader.blocks() {
        file.write(&block)
            .or_else(|e| backrub_error("Could not write to output file", Some(e.into())))?;
    }
    Ok(())
}

fn backup_blocks(blocks: FsBlockSource, object: &mut dyn BackupObjectWriter) -> Result<()> {
    for block in blocks {
        object.add_block(&block)?;
    }
    println!("Finished copying blocks");
    Ok(())
}
