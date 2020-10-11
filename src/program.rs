use super::backup::BackupInstance;
use super::backupobject::BackupObject;
use super::backupobject::BackupObjectWriter;
use super::crypto::{Cipher, CryptoBlock};
use super::errors::backrub_error;
use super::errors::Result;
use super::fsrepository::FsRepository;
use super::fssource::{FsBlockSource, FsSource};
use super::repository::Repository;
use crate::common::InternalCryptoBlock;
use rmp_serde::Deserializer;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

pub fn initialize_repository(repository: &str) -> Result<()> {
    let repo: FsRepository = Repository::new(&repository);
    repo.initialize()?;
    Ok(())
}
pub fn list_instances(repo: &str) -> Result<()> {
    let _: FsRepository = Repository::new(repo);
    Ok(())
}
