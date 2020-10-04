use super::backup::BackupInstance;
use super::backupobject::BackupObjectReader;
use super::backupobject::{BackupObject, BackupObjectWriter};
use super::errors::Result;
use super::repository::Repository;
use crate::errors::backrub_error;
use hex;
use rmp_serde::decode::Error;
use rmp_serde::Deserializer;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fs;
use std::fs::File;
use std::io;
use std::path;
use std::path::Path;

use rmp_serde::Serializer;

pub struct FsRepository {
    path: String,
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
        };
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
        }
        Ok(())
    }

    fn start_object(&self, name: &str) -> Result<Box<dyn BackupObjectWriter>> {
        println!("Starting new backup object: {}", name);
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
        println!("Finished writing instance {} to repository.", backup.name);
        Ok(())
    }
    fn open_object(&self, id: &str) -> Result<std::boxed::Box<dyn BackupObjectReader>> {
        let object_path: path::PathBuf =
            [&self.path, "blocks", &id[..2], &id[2..]].iter().collect();
        let file = fs::File::open(object_path)
            .or_else(|e| backrub_error("Could not open object", Some(e.into())))?;
        let mut deserializer = Deserializer::new(file);
        let meta = Deserialize::deserialize(&mut deserializer)
            .or_else(|e| backrub_error("Could not deserialize object", Some(e.into())))?;
        Ok(Box::new(FsBackupObject {
            repo_path: self.path.clone(),
            meta: meta,
        }))
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
        println!("Added block of size {} with id {}", data.len(), id);
        self.meta.blocks.push(id.clone());
        Ok(id)
    }

    fn finish(&self) -> Result<String> {
        let mut buf: Vec<u8> = Vec::new();
        self.meta
            .serialize(&mut Serializer::new(&mut buf))
            .or_else(|e| backrub_error("Could not serialize meta data", Some(e.into())))?;
        self.write_block(&buf)
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
    let meta = BackrubRepositoryMeta {
        version: 1,
        title: String::from("backrub backup repository."),
    };
    let meta_path: path::PathBuf = [&path, "backrub"].iter().collect();
    let file = &mut File::create(meta_path)
        .or_else(|e| backrub_error("Could not create repository marker file", Some(e.into())))?;
    meta.serialize(&mut Serializer::new(file));
    let block_path: path::PathBuf = [&path, "blocks"].iter().collect();
    fs::create_dir_all(block_path)
        .or_else(|e| backrub_error("Could not create block storage", Some(e.into())))?;
    let instance_path: path::PathBuf = [&path, "instances"].iter().collect();
    fs::create_dir_all(instance_path)
        .or_else(|e| backrub_error("Could not create instance storage", Some(e.into())))?;
    Ok(())
}

#[derive(Debug)]
pub struct BackrubBlock {
    pub id: String,
    pub data: Vec<u8>,
}

/**
 * The main header definition for a repository
 */
#[derive(Serialize, Deserialize)]
struct BackrubRepositoryMeta {
    version: u32,
    title: String,
}

/**
 * The definition of a specific backup instance
 */
#[derive(Serialize, Deserialize)]
struct BackrubBackup {
    name: String,       // the name of the backup. Must be unique.File
    root_block: String, // the root block ID of the backup tree
}
