use super::errors::Result;
use super::fsrepository::FsRepository;
use super::repository::Repository;
use crate::common::read_key;
use std::path::Path;

pub fn initialize_repository(repository: &str) -> Result<()> {
    let repo = FsRepository::new(&Path::new(&repository));
    let user_key = read_key()?;
    repo.initialize(user_key)?;
    Ok(())
}
