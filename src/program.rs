use super::errors::Result;
use super::fsrepository::FsRepository;
use super::repository::Repository;
use crate::common::read_key;

pub fn initialize_repository(repository: &str) -> Result<()> {
    let repo: FsRepository = Repository::new(&repository);
    let user_key = read_key()?;
    repo.initialize(user_key)?;
    Ok(())
}
pub fn list_instances(repo: &str) -> Result<()> {
    let _: FsRepository = Repository::new(repo);
    Ok(())
}
