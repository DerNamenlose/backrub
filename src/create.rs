use super::backup::BackupInstance;
use super::common::read_key;
use super::errors::backrub_error;
use super::errors::Result;
use super::fsrepository::FsRepository;
use super::fssource::FsSource;
use super::repository::Repository;
use crate::backup::BackupEntry;
use crate::backup::EntryList;
use crate::backup::EntryType;
use crate::backup::FileEntryData;
use crate::backup::LinkData;
use crate::backup::Meta;
use crate::backupobject::BackupObject;
use crate::blockcache;
use crate::blockcache::BlockCache;
use crate::crypto::encode_keyed_block;
use crate::crypto::DataEncryptionKey;
use crate::errors::Error;
use crate::fssource::FsBlockSource;
use crate::os::unix::get_meta_data;
use crate::repository::BackupBlockId;
use rmp_serde::Serializer;
use serde::Serialize;
use sha3::{Digest, Sha3_256};
use std::path::Path;
use std::time::SystemTime;

/**
 * entry point for the create sub-command
 */

pub fn make_backup(repository: &str, path: &str, cache_dir: &Path, name: &str) -> Result<()> {
    let mut repo = FsRepository::new(&Path::new(&repository));
    let cache = blockcache::open(&cache_dir)?;
    cache.ensure()?;
    let key = read_key()?;
    repo.open(key)?;
    let current_key = repo.current_key()?;
    let source: FsSource = FsSource::new(&path);

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Could not get current time");
    let mut backup_entries = EntryList::from(vec![]);
    let mut total_size: usize = 0;
    for object in source.objects() {
        let (entry, size) = backup_object(&path, &source, &repo, &cache, &current_key, object)?;
        backup_entries.0.push(entry);
        total_size += size;
    }
    log::info!("Finishing backup");
    let (entry_list_id, size) = repo.store_entry_list(&backup_entries)?;
    repo.finish_backup(BackupInstance {
        name: String::from(name),
        time: now.as_secs(),
        entry_list_id: entry_list_id,
    })
    .or_else(|e| backrub_error("Could not finish backup instance", Some(e.into())))?;
    total_size += size;
    log::info!("Finished backup");
    log::info!("Total backup size: {} bytes", total_size);
    Ok(())
}

fn backup_object(
    path: &str,
    source: &FsSource,
    repo: &FsRepository,
    cache: &impl BlockCache,
    current_key: &(u64, DataEncryptionKey),
    object: walkdir::DirEntry,
) -> Result<(BackupEntry, usize)> {
    let file_type = object.file_type();
    if file_type.is_file() {
        backup_file(path, source, repo, cache, current_key, object)
    } else if file_type.is_dir() {
        backup_dir(path, object)
    } else if file_type.is_symlink() {
        backup_link(path, object)
    } else {
        backrub_error("Unsupported object type", None)
    }
}

fn get_name(entry: &walkdir::DirEntry) -> Result<&str> {
    entry.path().to_str().ok_or(Error {
        message: "Could not decode file path",
        cause: None,
    })
}

fn get_relative_name<'a>(entry: &'a walkdir::DirEntry, base: &'a str) -> Result<&'a str> {
    entry
        .path()
        .strip_prefix(&base)
        .or_else(|e| backrub_error("Could not get relative source name", Some(e.into())))
        .and_then(|p| {
            p.to_str().ok_or(Error {
                message: "Could not decode file path",
                cause: None,
            })
        })
}

fn backup_file(
    path: &str,
    source: &FsSource,
    repo: &FsRepository,
    cache: &impl BlockCache,
    current_key: &(u64, DataEncryptionKey),
    file: walkdir::DirEntry,
) -> Result<(BackupEntry, usize)> {
    let source_name = get_name(&file)?;
    let source_name_relative = get_relative_name(&file, &path)?;
    let source_meta_data = get_meta_data(&file.path())?;
    let meta_block = get_meta_block(&source_name, &source_meta_data)?;
    if let Ok(Some(backup_id)) = cache.get_backup_block_id(&meta_block) {
        log::trace!("Block cache hit for \"{}\"", source_name);
        Ok((
            BackupEntry {
                name: String::from(source_name_relative),
                entry_type: EntryType::File(FileEntryData {
                    block_list_id: backup_id,
                }),
                meta: source_meta_data,
            },
            0,
        ))
    } else {
        log::trace!("Block cache miss for \"{}\"", source_name);
        let blocks = source.open_entry(&source_name)?;
        let mut object = BackupObject { blocks: vec![] };
        let mut size = 0;
        let block_sum = backup_blocks(blocks, &mut object, &repo, cache, &current_key)?;
        size += block_sum;
        log::debug!("Adding object descriptor to repository");
        let (id, descriptor_size) = finish_object(&object, &repo, &current_key)?;
        log::debug!("New object: {}", id);
        size += descriptor_size;
        cache.add_block(&meta_block, &id)?;
        Ok((
            BackupEntry {
                name: String::from(source_name_relative),
                entry_type: EntryType::File(FileEntryData { block_list_id: id }),
                meta: source_meta_data,
            },
            size,
        ))
    }
}

fn backup_dir(path: &str, dir: walkdir::DirEntry) -> Result<(BackupEntry, usize)> {
    let source_name_relative = get_relative_name(&dir, &path)?;
    Ok((
        BackupEntry {
            name: String::from(source_name_relative),
            entry_type: EntryType::Dir,
            meta: get_meta_data(dir.path())?,
        },
        0,
    ))
}

fn backup_link(path: &str, link: walkdir::DirEntry) -> Result<(BackupEntry, usize)> {
    let source_name_relative = get_relative_name(&link, &path)?;
    let link_target = std::fs::read_link(link.path())
        .or_else(|e| backrub_error("Could not read link target", Some(e.into())))?;
    let link_target_string = link_target.to_str().ok_or(Error {
        message: "Could not decode link target path",
        cause: None,
    })?;
    Ok((
        BackupEntry {
            name: String::from(source_name_relative),
            entry_type: EntryType::Link(LinkData {
                target: String::from(link_target_string),
            }),
            meta: get_meta_data(link.path())?,
        },
        0,
    ))
}

fn backup_blocks(
    blocks: FsBlockSource,
    object: &mut BackupObject,
    repo: &FsRepository,
    cache: &impl BlockCache,
    key: &(u64, DataEncryptionKey),
) -> Result<usize> {
    let mut stored_size = 0;
    for block in blocks {
        let backup_id = if let Ok(Some(backup_block_id)) = cache.get_backup_block_id(&block) {
            log::trace!("Block cache hit for {}", backup_block_id);
            backup_block_id
        } else {
            let mut output_block = vec![];
            encode_keyed_block(&mut output_block, &block, &key)?;
            let (id, size) = repo.add_block(&output_block)?;
            log::trace!("Block cache miss for {}", id);
            stored_size += size;
            id
        };
        object.blocks.push(backup_id);
    }
    log::debug!("Finished copying blocks");
    Ok(stored_size)
}

fn finish_object(
    object: &BackupObject,
    repo: &FsRepository,
    current_key: &(u64, DataEncryptionKey),
) -> Result<(BackupBlockId, usize)> {
    let mut object_buffer = vec![];
    (*object)
        .serialize(&mut Serializer::new(&mut object_buffer))
        .or_else(|e| backrub_error("Could not serialize meta data", Some(e.into())))?;
    let mut storage_buf = vec![];
    encode_keyed_block(&mut storage_buf, &object_buffer, current_key)?;
    let (id, size) = repo.add_block(&storage_buf)?;
    Ok((id, size))
}

fn get_meta_block(path: &str, meta: &Meta) -> Result<Vec<u8>> {
    let mut buf = vec![];
    let mut serializer = Serializer::new(&mut buf);
    (*path)
        .serialize(&mut serializer)
        .or_else(|e| backrub_error("Could not serialize path", Some(e.into())))?;
    (*meta)
        .serialize(&mut serializer)
        .or_else(|e| backrub_error("Could not serialize meta", Some(e.into())))?;
    let mut hasher = Sha3_256::new();
    hasher.update(&buf);
    Ok(hasher.finalize().to_vec())
}

// fn get_file_meta(entry: walkdir::DirEntry) -> Result<Meta> {}
