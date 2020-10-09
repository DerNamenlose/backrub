use super::backup::BackupInstance;
use super::backupobject::BackupObjectReader;
use super::backupobject::{BackupObject, BackupObjectWriter};
use super::errors::backrub_error;
use super::errors::Result;
use super::repository::Repository;
use crate::repository::BackrubRepositoryMeta;
use hex;
use log;
use rand::rngs;
use rand::RngCore;
use rmp_serde::Deserializer;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fs;
use std::fs::File;
use std::path;

use rmp_serde::Serializer;

pub struct FsRepository {
    path: String,
    repo_info: Option<BackrubRepositoryMeta>,
}

fn path_for(base: &str, segments: &[&str]) -> path::PathBuf {
    let mut p = path::PathBuf::from(&base);
    for seg in segments {
        p.push(seg);
    }
    p
}

impl Repository for FsRepository {
    fn new(path: &str) -> Self {
        return FsRepository {
            path: String::from(path),
            repo_info: None,
        };
    }
    fn meta(&self) -> Result<&BackrubRepositoryMeta> {
        match &self.repo_info {
            Some(info) => Ok(info),
            None => backrub_error(
                "Internal error. Tried to access meta data before repository was loaded",
                None,
            ),
        }
    }
    fn initialize(&self) -> Result<()> {
        fs::create_dir_all(&self.path).or_else(|e| {
            backrub_error(
                "Could not create backup repository directory",
                Some(e.into()),
            )
        })?;
        if !is_initialized(&self.path) {
            create_backrub_infrastructure(&self.path)?;
            Ok(())
        } else {
            backrub_error(
                "Cannot initialize already initialized repository again.",
                None,
            )
        }
    }
    fn open(&mut self) -> Result<()> {
        let meta_path: path::PathBuf = [&self.path, "backrub"].iter().collect();
        let f = fs::File::open(meta_path).or_else(|e| {
            backrub_error(
                "Could not open meta file. Is this a backrub repository?",
                Some(e.into()),
            )
        })?;
        let mut deserializer = Deserializer::new(f);
        let meta: BackrubRepositoryMeta =
            Deserialize::deserialize(&mut deserializer).or_else(|e| {
                backrub_error("Could not deserialize repository meta data", Some(e.into()))
            })?;
        self.repo_info = Some(meta);
        Ok(())
    }
    fn start_object(&self, name: &str) -> Result<Box<dyn BackupObjectWriter>> {
        log::debug!("Starting new backup object: {}", name);
        return Ok(Box::new(FsBackupObject {
            meta: BackupObject {
                name: String::from(name),
                blocks: vec![],
            },
            repo_path: String::from(&self.path),
        }));
    }
    fn finish_backup(&self, backup: BackupInstance) -> Result<()> {
        let backup_file_name = format!("{}-{}", backup.time, backup.name);
        let instance_path = path_for(&self.path, &["instances", &backup_file_name]);
        let file = fs::File::create(instance_path)
            .or_else(|e| backrub_error("Could not create instance file", Some(e.into())))?;
        backup
            .serialize(&mut Serializer::new(file))
            .or_else(|e| backrub_error("Could not serialize instance", Some(e.into())))?;
        log::info!("Finished writing instance {} to repository.", backup.name);
        Ok(())
    }
    fn open_object(&self, id: &str) -> Result<BackupObject> {
        let object_path: path::PathBuf =
            [&self.path, "blocks", &id[..2], &id[2..]].iter().collect();
        let file = fs::File::open(object_path)
            .or_else(|e| backrub_error("Could not open object", Some(e.into())))?;
        let mut deserializer = Deserializer::new(file);
        Deserialize::deserialize(&mut deserializer)
            .or_else(|e| backrub_error("Could not deserialize object", Some(e.into())))
    }
    fn open_object_reader(
        &self,
        meta: BackupObject,
    ) -> Result<std::boxed::Box<dyn BackupObjectReader>> {
        Ok(Box::new(FsBackupObject {
            repo_path: self.path.clone(),
            meta: meta,
        }))
    }
    fn list_instances(&self) -> Result<Vec<String>> {
        let instance_path: path::PathBuf = [&self.path, "instances"].iter().collect();
        let entries = fs::read_dir(instance_path)
            .or_else(|e| backrub_error("Could not open backup instances", Some(e.into())))?;
        // Ok(Vec::from(
        //     entries.filter_map(|entry| entry.ok()?.path().to_str()),
        // ))
        Ok(vec![])
    }
    fn open_instance(&self, name: &str) -> Result<BackupInstance> {
        let instance_path: path::PathBuf = [&self.path, "instances", name].iter().collect();
        let p = instance_path.to_str();
        match p {
            Some(ip) => {
                let file = fs::File::open(ip)
                    .or_else(|e| backrub_error("Could not open instance", Some(e.into())))?;
                let mut deserializer = Deserializer::new(file);
                let instance = Deserialize::deserialize(&mut deserializer)
                    .or_else(|e| backrub_error("Could not deserialize object", Some(e.into())))?;
                Ok(instance)
            }
            None => backrub_error("Could not serialize instance path", None),
        }
    }
}

pub struct FsBackupObject {
    meta: BackupObject,
    repo_path: String,
}

impl FsBackupObject {
    fn write_block(&self, data: &[u8]) -> Result<String> {
        let mut hasher = Sha3_256::new();
        hasher.update(&data);
        let id = hex::encode(hasher.finalize());
        let prefix = &id[..2];
        let parent_path: path::PathBuf = path_for(&self.repo_path, &["blocks", prefix]);
        let data_path: path::PathBuf = path_for(&self.repo_path, &["blocks", prefix, &id[2..]]);
        fs::create_dir_all(parent_path)
            .or_else(|e| backrub_error("Could not create parent directory", Some(e.into())))?;
        fs::write(data_path, &data)
            .or_else(|e| backrub_error("Could not write file", Some(e.into())))?;
        Ok(String::from(&id))
    }
}

impl BackupObjectWriter for FsBackupObject {
    fn add_block(&mut self, data: &[u8]) -> Result<std::string::String> {
        let id = self.write_block(data)?;
        log::debug!("Added block of size {} with id {}", data.len(), id);
        self.meta.blocks.push(id.clone());
        Ok(id)
    }

    fn finish(&self) -> Result<String> {
        let mut buf: Vec<u8> = Vec::new();
        self.meta
            .serialize(&mut Serializer::new(&mut buf))
            .or_else(|e| backrub_error("Could not serialize meta data", Some(e.into())))?;
        let result = self.write_block(&buf)?;
        log::info!("Stored {} in repository", self.meta.name);
        Ok(result)
    }
}

struct FsBackupObjectBlockSource<'a> {
    path: String,
    block_iter: std::slice::Iter<'a, String>,
}

impl<'a> FsBackupObjectBlockSource<'a> {
    pub fn new(path: String, iter: std::slice::Iter<'a, String>) -> Self {
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
        let block_path: path::PathBuf = [&self.path, "blocks", &block_id[..2], &block_id[2..]]
            .iter()
            .collect();
        fs::read(block_path).ok()
    }
}

impl BackupObjectReader for FsBackupObject {
    fn blocks<'a>(&'a self) -> Box<dyn Iterator<Item = Vec<u8>> + 'a> {
        Box::new(FsBackupObjectBlockSource::new(
            String::from(&self.repo_path),
            self.meta.blocks.iter(),
        ))
    }
}

fn is_initialized(path: &str) -> bool {
    let meta_path: path::PathBuf = [&path, "backrub"].iter().collect();
    meta_path.exists()
}

fn create_backrub_infrastructure(path: &str) -> Result<()> {
    let (iterations, salt) = initialize_key_derivation("SomeRandomCalibrationKey");
    let meta = BackrubRepositoryMeta {
        version: 1,
        title: String::from("backrub backup repository."),
        salt: salt,
        iterations: iterations,
    };
    let meta_path: path::PathBuf = [&path, "backrub"].iter().collect();
    let file = &mut File::create(meta_path)
        .or_else(|e| backrub_error("Could not create repository marker file", Some(e.into())))?;
    meta.serialize(&mut Serializer::new(file))
        .or_else(|e| backrub_error("Could not serialize repository marker", Some(e.into())))?;
    let block_path: path::PathBuf = [&path, "blocks"].iter().collect();
    fs::create_dir_all(block_path)
        .or_else(|e| backrub_error("Could not create block storage", Some(e.into())))?;
    let instance_path: path::PathBuf = [&path, "instances"].iter().collect();
    fs::create_dir_all(instance_path)
        .or_else(|e| backrub_error("Could not create instance storage", Some(e.into())))?;
    Ok(())
}

fn initialize_key_derivation(key: &str) -> (u16, Vec<u8>) {
    log::debug!("Calibrating key derivation function");
    let mut time_factor: u16 = 3;
    let mut salt = [0; 16];
    rngs::OsRng.fill_bytes(&mut salt);
    loop {
        let start = std::time::Instant::now();
        let _ = super::crypto::derive_key(key.as_ref(), &salt, time_factor as u32);
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
