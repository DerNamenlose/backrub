use super::backupobject::BackupObject;
use super::common::read_key;
use super::crypto::Cipher;
use super::errors::backrub_error;
use super::errors::Result;
use super::fsrepository::FsRepository;
use super::repository::Repository;
use crate::common::InternalCryptoBlock;
use crate::crypto::CryptoBlock;
use rmp_serde::Deserializer;
use serde::Deserialize;
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
    let current_key = repository.current_key()?;
    let cipher = Cipher::new(current_key);
    let instance = repository.open_instance(name)?;
    let mut errors = vec![];
    for entry in instance.entries {
        let object_result = repository.open_object(&entry);
        match object_result {
            Ok(object) => {
                let restore_result = restore_object(&repository, object, path, &cipher);
                match restore_result {
                    Ok(_) => log::debug!("Successfully restored object"),
                    Err(e) => errors.push(e),
                }
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }
    if errors.len() != 0 {
        log::error!("{} error(s) occured during restore", errors.len());
        for error in errors {
            log::error!("{}", error);
        }
        backrub_error("Restore unsuccessful", None)
    } else {
        Ok(())
    }
}

fn restore_object(
    repo: &FsRepository,
    object: BackupObject,
    base_path: &str,
    cipher: &Cipher,
) -> Result<()> {
    let restore_path: std::path::PathBuf = [base_path, &object.name].iter().collect();
    let parent_path = restore_path.parent().ok_or(super::errors::Error {
        message: "Object has no parent directory",
        cause: None,
    })?;
    log::debug!(
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
        log::debug!("Decoding serialized data block of size {}", block.len());
        let data_block = decode_block(block, cipher)?;
        log::debug!("Contained block of size {}", data_block.len());
        file.write(&data_block)
            .or_else(|e| backrub_error("Could not write to output file", Some(e.into())))?;
    }
    Ok(())
}

fn decode_block(block: Vec<u8>, cipher: &Cipher) -> Result<Vec<u8>> {
    let mut deserializer = Deserializer::new(std::io::Cursor::new(block));
    let instance: InternalCryptoBlock = Deserialize::deserialize(&mut deserializer)
        .or_else(|e| backrub_error("Could not deserialize object", Some(e.into())))?;
    let crypto_block = CryptoBlock {
        nonce: instance.nonce.to_vec(),
        data: instance.data.to_vec(),
    };
    cipher.decrypt_block(crypto_block)
}
