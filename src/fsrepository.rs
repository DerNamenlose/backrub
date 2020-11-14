use super::backup::BackupInstance;
use super::backupobject::BackupObject;
use super::backupobject::BackupObjectReader;
use super::errors::backrub_error;
use super::errors::Result;
use super::repository::Repository;
use crate::backup::EntryList;
use crate::crypto::decode_keyed_block;
use crate::crypto::derive_key;
use crate::crypto::encode_keyed_block;
use crate::crypto::Cipher;
use crate::crypto::CryptoBlock;
use crate::crypto::DataEncryptionKey;
use crate::crypto::InputKey;
use crate::crypto::MasterKey;
use crate::repository::BackrubRepositoryMeta;
use crate::repository::BackupBlockId;
use hex;
use log;
use rand::rngs;
use rand::RngCore;
use rmp_serde::Deserializer;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::path;
use std::path::Path;
use std::path::PathBuf;

use rmp_serde::Serializer;

pub struct FsRepository<'a> {
    path: &'a Path,
    repo_info: Option<BackrubRepositoryMeta>,
    keys: HashMap<u64, DataEncryptionKey>,
    current_key: Option<(u64, DataEncryptionKey)>,
}

fn path_for(base: &Path, segments: &[&str]) -> path::PathBuf {
    let mut p = path::PathBuf::from(&base);
    for seg in segments {
        p.push(seg);
    }
    p
}

impl FsRepository<'_> {
    pub fn new<'a>(path: &'a Path) -> FsRepository<'a> {
        return FsRepository {
            path: path,
            repo_info: None,
            keys: HashMap::new(),
            current_key: None,
        };
    }
    fn open_instance_file(path: &Path) -> Result<BackupInstance> {
        let file = fs::File::open(path)
            .or_else(|e| backrub_error("Could not open instance", Some(e.into())))?;
        let mut deserializer = Deserializer::new(file);
        let instance = Deserialize::deserialize(&mut deserializer)
            .or_else(|e| backrub_error("Could not deserialize instance", Some(e.into())))?;
        Ok(instance)
    }
}

impl Repository for FsRepository<'_> {
    fn meta(&self) -> Result<&BackrubRepositoryMeta> {
        match &self.repo_info {
            Some(info) => Ok(info),
            None => backrub_error(
                "Internal error. Tried to access meta data before repository was loaded",
                None,
            ),
        }
    }
    fn initialize(&self, input_key: InputKey) -> Result<()> {
        fs::create_dir_all(&self.path).or_else(|e| {
            backrub_error(
                "Could not create backup repository directory",
                Some(e.into()),
            )
        })?;
        if !is_initialized(&self.path) {
            create_backrub_infrastructure(&self.path, &input_key)?;
            Ok(())
        } else {
            backrub_error(
                "Cannot initialize already initialized repository again.",
                None,
            )
        }
    }
    fn open(&mut self, input_key: InputKey) -> Result<()> {
        let ri = load_meta_data(&self.path)?;
        let keys = load_keys(
            &self.path,
            &derive_key(&input_key, &ri.salt, ri.iterations as u32)?,
        )?;
        self.repo_info = Some(ri);
        let mut key_map = HashMap::new();
        for key in keys {
            key_map.insert(key.0, key.1);
        }
        self.keys = key_map;
        let current = self
            .keys
            .iter()
            .min_by(|a, b| a.1.created_at.cmp(&b.1.created_at))
            .unwrap()
            .clone();
        self.current_key = Some((*current.0, (*current.1).clone()));
        Ok(())
    }
    // fn start_object(&self, name: &str) -> Result<Box<dyn BackupObjectWriter>> {
    //     log::debug!("Starting new backup object: {}", name);
    //     return Ok(Box::new(FsBackupObject {
    //         meta: BackupObject { blocks: vec![] },
    //         entry: BackupEntry {
    //             name: String::from(name),
    //             block_list_id: String::new(),
    //         },
    //         repo_path: String::from(&self.path),
    //     }));
    // }

    fn add_block(&self, data: &[u8]) -> Result<(BackupBlockId, usize)> {
        let id = write_block(&self.path, data)?;
        log::debug!("Added block of size {} with id {}", data.len(), id);
        Ok((id, data.len()))
    }

    fn store_entry_list(&self, entries: &EntryList) -> Result<(BackupBlockId, usize)> {
        let mut output_block = vec![];
        entries
            .serialize(&mut Serializer::new(&mut output_block))
            .or_else(|e| backrub_error("Could not serialize entry list", Some(e.into())))?;
        let mut target_block = vec![];
        let current_key = self.current_key()?;
        encode_keyed_block(&mut target_block, &output_block, current_key)?;
        self.add_block(&target_block)
    }

    fn load_entry_list(&self, entry_list_id: &BackupBlockId) -> Result<EntryList> {
        let id_str = entry_list_id.to_str();
        let block_path = path_for(&self.path, &["blocks", &id_str[..2], &id_str[2..]]);
        let block_file = fs::File::open(&block_path)
            .or_else(|e| backrub_error("Could not read entry list block", Some(e.into())))?;
        let keyset = self.keys()?;
        let decoded_block = decode_keyed_block(block_file, &keyset)?;
        let mut list_deserializer = Deserializer::new(Cursor::new(&decoded_block));
        Deserialize::deserialize(&mut list_deserializer)
            .or_else(|e| backrub_error("Could not deserialize entry list", Some(e.into())))
    }
    fn finish_backup(&self, backup: BackupInstance) -> Result<()> {
        let instance_path = path_for(&self.path, &["instances", &backup.name]);
        let file = fs::File::create(instance_path)
            .or_else(|e| backrub_error("Could not create instance file", Some(e.into())))?;
        backup
            .serialize(&mut Serializer::new(file))
            .or_else(|e| backrub_error("Could not serialize instance", Some(e.into())))?;
        log::info!("Finished writing instance {} to repository.", backup.name);
        Ok(())
    }
    fn open_object(&self, id: &BackupBlockId) -> Result<BackupObject> {
        let id_str = id.to_str();
        let file = fs::File::open(
            self.path
                .join("blocks")
                .join(&id_str[..2])
                .join(&id_str[2..]),
        )
        .or_else(|e| backrub_error("Could not open object", Some(e.into())))?;
        let keys = self.keys()?;
        let decoded_block = decode_keyed_block(file, &keys)?;
        Deserialize::deserialize(&mut Deserializer::new(&mut Cursor::new(&decoded_block)))
            .or_else(|e| backrub_error("Could not deserialize object", Some(e.into())))
    }
    fn open_object_reader(&self, meta: BackupObject) -> Result<Box<dyn BackupObjectReader>> {
        Ok(Box::new(FsBackupObjectReader {
            repo_path: self.path.to_owned(),
            meta: meta,
        }))
    }
    fn list_instances(&self) -> Result<Vec<BackupInstance>> {
        let entries = fs::read_dir(self.path.join("instances"))
            .or_else(|e| backrub_error("Could not open backup instances", Some(e.into())))?;
        let result = entries.filter_map(|entry| entry.ok().map(|e| e.path()));
        let instances = result.filter_map(|p| FsRepository::open_instance_file(&p).ok());
        Ok(instances.collect())
    }
    fn open_instance(&self, name: &str) -> Result<BackupInstance> {
        FsRepository::open_instance_file(&self.path.join("instances").join(name))
    }
    fn keys(&self) -> Result<&HashMap<u64, DataEncryptionKey>> {
        Ok(&self.keys)
    }
    fn current_key(&self) -> Result<&(u64, DataEncryptionKey)> {
        if let Some(key) = &self.current_key {
            Ok(key)
        } else {
            backrub_error("current key not loaded", None)
        }
    }
}

fn load_meta_data(base_path: &Path) -> Result<BackrubRepositoryMeta> {
    let f = fs::File::open(base_path.join("backrub")).or_else(|e| {
        backrub_error(
            "Could not open meta file. Is this a backrub repository?",
            Some(e.into()),
        )
    })?;
    let mut deserializer = Deserializer::new(f);
    let meta: BackrubRepositoryMeta = Deserialize::deserialize(&mut deserializer)
        .or_else(|e| backrub_error("Could not deserialize repository meta data", Some(e.into())))?;
    Ok(meta)
}

fn load_keys(base_path: &Path, master_key: &MasterKey) -> Result<Vec<(u64, DataEncryptionKey)>> {
    let key_entries = fs::read_dir(base_path.join("keys"))
        .or_else(|err| backrub_error("Could not read key storage", Some(err.into())))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            if let Ok(ft) = e.file_type() {
                ft.is_file()
            } else {
                false
            }
        })
        .map(|e| e.path());
    key_entries
        .map(|p| read_data_encryption_key(&p, master_key))
        .collect::<Result<Vec<(u64, DataEncryptionKey)>>>()
}

fn write_block(repo_path: &Path, data: &[u8]) -> Result<BackupBlockId> {
    let mut hasher = Sha3_256::new();
    hasher.update(&data);
    let id_bytes = hasher.finalize();
    let id = hex::encode(&id_bytes);
    let prefix = &id[..2];
    let parent_path: path::PathBuf = path_for(&repo_path, &["blocks", prefix]);
    let data_path: path::PathBuf = path_for(&repo_path, &["blocks", prefix, &id[2..]]);
    fs::create_dir_all(parent_path)
        .or_else(|e| backrub_error("Could not create parent directory", Some(e.into())))?;
    fs::write(data_path, &data)
        .or_else(|e| backrub_error("Could not write file", Some(e.into())))?;
    BackupBlockId::from_bytes(&id_bytes)
}

struct FsBackupObjectBlockSource<'a> {
    path: &'a Path,
    block_iter: std::slice::Iter<'a, BackupBlockId>,
}

impl<'a> FsBackupObjectBlockSource<'a> {
    pub fn new(path: &'a Path, iter: std::slice::Iter<'a, BackupBlockId>) -> Self {
        FsBackupObjectBlockSource {
            path: path,
            block_iter: iter,
        }
    }
}

impl<'a> Iterator for FsBackupObjectBlockSource<'a> {
    type Item = Vec<u8>;

    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {
        let block_id = self.block_iter.next()?;
        let id_str = block_id.to_str();
        fs::read(
            self.path
                .join("blocks")
                .join(&id_str[..2])
                .join(&id_str[2..]),
        )
        .ok()
    }
}

pub struct FsBackupObjectReader {
    meta: BackupObject,
    repo_path: PathBuf,
}

impl BackupObjectReader for FsBackupObjectReader {
    fn blocks<'a>(&'a self) -> Box<dyn Iterator<Item = Vec<u8>> + 'a> {
        Box::new(FsBackupObjectBlockSource::new(
            &self.repo_path,
            self.meta.blocks.iter(),
        ))
    }
}

fn is_initialized(path: &Path) -> bool {
    path.join("backrub").exists()
}

fn create_backrub_infrastructure(path: &Path, master_password: &InputKey) -> Result<()> {
    log::debug!("Initialize key derivation");
    let (iterations, salt) = initialize_key_derivation();
    let meta = BackrubRepositoryMeta {
        version: 1,
        title: String::from("backrub backup repository."),
        salt: salt,
        iterations: iterations,
    };
    log::debug!("Creating main meta file");
    let file = &mut File::create(path.join("backrub"))
        .or_else(|e| backrub_error("Could not create repository marker file", Some(e.into())))?;
    meta.serialize(&mut Serializer::new(file))
        .or_else(|e| backrub_error("Could not serialize repository marker", Some(e.into())))?;
    log::debug!("Creating block storage");
    fs::create_dir_all(path.join("blocks"))
        .or_else(|e| backrub_error("Could not create block storage", Some(e.into())))?;
    log::debug!("Creating instance storage");
    fs::create_dir_all(path.join("instances"))
        .or_else(|e| backrub_error("Could not create instance storage", Some(e.into())))?;
    log::debug!("Creating key storage");
    fs::create_dir_all(path.join("keys"))
        .or_else(|e| backrub_error("Could not create key storage", Some(e.into())))?;
    log::debug!("Creating initial data encryption key");
    let master_key = derive_key(&master_password, &meta.salt, iterations as u32)?;
    create_data_encryption_key(&path, master_key)?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct EncryptedDataEncryptionKey {
    pub created_at: u64,        // UNIX timestamp of the key generation
    pub key_block: CryptoBlock, // The encrypted data encryption key
}

fn create_data_encryption_key(path: &Path, master_key: MasterKey) -> Result<()> {
    let mut key_bytes = [0; 32];
    rngs::OsRng.fill_bytes(&mut key_bytes);
    let key_file_name = format!("{:016x}.key", rand::thread_rng().next_u64());
    let key_file_path = path.join("keys").join(&key_file_name);
    let cipher = Cipher::new(&DataEncryptionKey::from(&master_key));
    let encrypted_key_block = cipher.encrypt_block(&Vec::from(key_bytes))?;
    let key_file = fs::File::create(key_file_path)
        .or_else(|e| backrub_error("Could not open key file", Some(e.into())))?;
    let current_unix_time = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .or_else(|e| {
            backrub_error(
                "Could not get the current time as a UNIX timestamp",
                Some(e.into()),
            )
        })?
        .as_secs();
    let key_storage = EncryptedDataEncryptionKey {
        created_at: current_unix_time,
        key_block: encrypted_key_block,
    };
    key_storage
        .serialize(&mut Serializer::new(key_file))
        .or_else(|e| backrub_error("Could not store data encryption key", Some(e.into())))?;
    Ok(())
}

fn read_data_encryption_key(
    p: &path::Path,
    master_key: &MasterKey,
) -> Result<(u64, DataEncryptionKey)> {
    let key_file =
        File::open(&p).or_else(|e| backrub_error("Could not read key file", Some(e.into())))?;
    let cipher = Cipher::new(&DataEncryptionKey::from(master_key));
    let mut deserializer = Deserializer::new(key_file);
    let encrypted_key: EncryptedDataEncryptionKey = Deserialize::deserialize(&mut deserializer)
        .or_else(|e| backrub_error("Could not deserialize key block", Some(e.into())))?;
    let key = cipher.decrypt_block(&encrypted_key.key_block)?;
    let key_index = if let Some(key_index_str) = p.file_stem() {
        Ok(u64::from_str_radix(key_index_str.to_str().unwrap(), 16)
            .or_else(|e| backrub_error("Could not parse key index", Some(e.into())))?)
    } else {
        backrub_error("Key file name has wrong format", None)
    }?;
    Ok((
        key_index,
        DataEncryptionKey {
            value: key,
            created_at: encrypted_key.created_at,
        },
    ))
}

fn initialize_key_derivation() -> (u16, Vec<u8>) {
    log::debug!("Calibrating key derivation function");
    let mut time_factor: u16 = 3;
    let mut salt = [0; 16];
    rngs::OsRng.fill_bytes(&mut salt);
    let test_key = InputKey::from(b"SomeRandomCalibrationKey" as &[u8]);
    loop {
        let start = std::time::Instant::now();
        let _ = super::crypto::derive_key(&test_key, &salt, time_factor as u32);
        let time = std::time::Instant::now().duration_since(start);
        let current_time_factor = 1000.0 / time.as_millis() as f32;
        if time.as_millis() > 1000 {
            break;
        }
        if current_time_factor < 1.1 {
            time_factor += 1;
        } else {
            time_factor = ((time_factor as f32) * current_time_factor) as u16;
        }
    }
    log::debug!("Iterations {}", time_factor);
    (time_factor, Vec::from(salt))
}

#[derive(Debug)]
pub struct BackrubBlock {
    pub id: String,
    pub data: Vec<u8>,
}

/**
 * The definition of a specific backup instance
 */
#[derive(Serialize, Deserialize)]
struct BackrubBackup {
    name: String,       // the name of the backup. Must be unique.File
    root_block: String, // the root block ID of the backup tree
}
