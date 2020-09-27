use crate::backupobject::{BackupObject, BackupObjectWriter};
use hex;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fs;
use std::fs::File;
use std::io;
use std::path;

use rmp_serde::Serializer;

pub trait Repository {
    /**
     * reate a new repository instance pointing to the given path
     */
    fn new(path: &str) -> Self;
    /**
     * Initialize the given backup repository, i.e. check whether anything needs to be set up in the
     * directory
     */
    fn initialize(&self) -> io::Result<()>;

    /**
     * Start a new backup object in the repository
     */
    fn start_object(&self, name: &str) -> std::result::Result<Box<dyn BackupObjectWriter>, String>;
}

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
    fn initialize(&self) -> std::result::Result<(), std::io::Error> {
        fs::create_dir_all(&self.path)?;
        if !is_initialized(&self.path) {
            create_backrub_infrastructure(&self.path)?;
        }
        Ok(())
    }

    fn start_object(&self, name: &str) -> std::result::Result<Box<dyn BackupObjectWriter>, String> {
        println!("Starting new backup object: {}", name);
        return Ok(Box::new(FsBackupObjectWriter {
            meta: BackupObject {
                name: String::from(name),
                blocks: vec![],
            },
            repo_path: String::from(&self.path),
        }));
    }
}

pub struct FsBackupObjectWriter {
    meta: BackupObject,
    repo_path: String,
}

impl FsBackupObjectWriter {
    fn write_block(&self, data: &[u8]) -> Result<String, &'static str> {
        let mut hasher = Sha3_256::new();
        hasher.update(&data);
        let id = hex::encode(hasher.finalize());
        let prefix = &id[..2];
        let parent_path: path::PathBuf = path_for(&self.repo_path, &["blocks", prefix]);
        let data_path: path::PathBuf = path_for(&self.repo_path, &["blocks", prefix, &id[2..]]);
        fs::create_dir_all(parent_path)
            .or(Err("Could not create parent directory"))
            .map(|()| fs::write(data_path, &data))
            .or(Err("Could not write file"))
            .and(Ok(String::from(&id)))
    }
}

impl BackupObjectWriter for FsBackupObjectWriter {
    fn add_block(&mut self, data: &[u8]) -> std::result::Result<std::string::String, &'static str> {
        let id = self.write_block(data)?;
        println!("Added block of size {} with id {}", data.len(), id);
        self.meta.blocks.push(id.clone());
        Ok(id)
    }

    fn finish(&self) -> Result<String, &'static str> {
        let mut buf: Vec<u8> = Vec::new();
        let serialization_result = self.meta.serialize(&mut Serializer::new(&mut buf));
        match serialization_result {
            Err(error) => {
                println!("Could not serialize meta data {}", error);
                Err("Could not serialize meta data")
            }
            Ok(()) => self.write_block(&buf),
        }
    }
}

fn is_initialized(path: &str) -> bool {
    let meta_path: path::PathBuf = [&path, "backrub"].iter().collect();
    meta_path.exists()
}

fn create_backrub_infrastructure(path: &str) -> io::Result<()> {
    let meta = BackrubRepositoryMeta {
        version: 1,
        title: String::from("backrub backup repository."),
    };
    let meta_path: path::PathBuf = [&path, "backrub"].iter().collect();
    let file = &mut File::create(meta_path)?;
    meta.serialize(&mut Serializer::new(file));
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
