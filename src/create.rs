use super::backup::BackupInstance;
use super::common::read_key;
use super::crypto::Cipher;
use super::errors::backrub_error;
use super::errors::Result;
use super::fsrepository::FsRepository;
use super::fssource::FsSource;
use super::repository::Repository;
use crate::backupobject::BackupObjectWriter;
use crate::common::InternalCryptoBlock;
use crate::fssource::FsBlockSource;
use rmp_serde::Serializer;
use serde::Serialize;
use std::time::SystemTime;

/**
 * entry point for the create sub-command
 */

pub fn make_backup(repository: &str, path: &str, name: &str) -> Result<()> {
    let mut repo: FsRepository = Repository::new(&repository);
    let key = read_key()?;
    repo.open(key)?;
    let cipher = Cipher::new(repo.current_key()?);
    let source: FsSource = FsSource::new(&path);

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
                    let copy_result = backup_blocks(blocks, object.as_mut(), &cipher);
                    match copy_result {
                        Ok(()) => {
                            log::debug!("Adding object descriptor to repository");
                            let finish_result = object.finish();
                            match finish_result {
                                Ok(id) => {
                                    log::debug!("New object: {}", id);
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
                (_, _) => log::error!("Could not copy source blocks into target object"),
            }
        }
    }
    log::info!("Finishing backup");
    repo.finish_backup(backup_instance)
        .or_else(|e| backrub_error("Could not finish backup instance", Some(e.into())))?;
    log::info!("Finished backup");
    Ok(())
}

fn backup_blocks(
    blocks: FsBlockSource,
    object: &mut dyn BackupObjectWriter,
    cipher: &Cipher,
) -> Result<()> {
    for block in blocks {
        let crypto_block = cipher.encrypt_block(block)?;
        let mut serialized_block = vec![];
        let write_test = InternalCryptoBlock {
            data: serde_bytes::ByteBuf::from(crypto_block.data),
            nonce: serde_bytes::ByteBuf::from(crypto_block.nonce),
        };
        write_test
            .serialize(&mut Serializer::new(&mut serialized_block))
            .or_else(|e| backrub_error("Could not serialize crypto block", Some(e.into())))?;
        object.add_block(&serialized_block)?;
    }
    log::debug!("Finished copying blocks");
    Ok(())
}
