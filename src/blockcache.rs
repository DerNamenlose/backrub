use crate::errors::backrub_error;
use crate::errors::Result;
use crate::repository::BackupBlockId;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

pub trait BlockCache {
    fn add_block(&self, data: &Vec<u8>, backup_block_id: &BackupBlockId) -> Result<()>;

    fn get_backup_block_id(&self, data: &Vec<u8>) -> Result<Option<BackupBlockId>>;
}

struct BlockCacheImpl<'a> {
    path: &'a str,
}

impl BlockCache for BlockCacheImpl<'_> {
    fn add_block(&self, data: &Vec<u8>, backup_block_id: &BackupBlockId) -> Result<()> {
        let mut hasher = Sha3_256::new();
        hasher.update(&data);
        let id_bytes = hasher.finalize();
        let id_str = hex::encode(id_bytes);
        let path: std::path::PathBuf = [self.path, &id_str].iter().collect();
        let id_file = std::fs::File::create(path)
            .or_else(|e| backrub_error("Could not write block to block index", Some(e.into())))?;
        backup_block_id
            .serialize(&mut Serializer::new(id_file))
            .or_else(|e| {
                backrub_error(
                    "Could not store backup block ID in block index",
                    Some(e.into()),
                )
            })?;
        Ok(())
    }

    fn get_backup_block_id(&self, data: &Vec<u8>) -> Result<Option<BackupBlockId>> {
        let mut hasher = Sha3_256::new();
        hasher.update(&data);
        let id_bytes = hasher.finalize();
        let id_str = hex::encode(id_bytes);
        let path: std::path::PathBuf = [self.path, &id_str].iter().collect();
        if let Ok(block_file) = std::fs::File::open(path) {
            let bid =
                Deserialize::deserialize(&mut Deserializer::new(block_file)).or_else(|e| {
                    backrub_error("Could not deserialize backup block ID", Some(e.into()))
                })?;
            Ok(Some(bid))
        } else {
            Ok(None)
        }
    }
}

pub fn open<'a>(path: &'a str) -> Result<impl BlockCache + 'a> {
    Ok(BlockCacheImpl { path: path })
}
