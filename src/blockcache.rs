use crate::errors::error;
use crate::errors::Result;
use crate::repository::BackupBlockId;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::path::Path;

pub trait BlockCache {
    /**
     * Ensure that the block cache exists
     */
    fn ensure(&self) -> Result<()>;

    fn add_block(&self, data: &Vec<u8>, backup_block_id: &BackupBlockId) -> Result<()>;

    fn get_backup_block_id(&self, data: &Vec<u8>) -> Result<Option<BackupBlockId>>;
}

struct BlockCacheImpl<'a> {
    path: &'a Path,
}

impl BlockCache for BlockCacheImpl<'_> {
    fn ensure(&self) -> Result<()> {
        std::fs::create_dir_all(self.path)
            .or_else(|e| error("Could not create block cache directory", Some(e.into())))
    }

    fn add_block(&self, data: &Vec<u8>, backup_block_id: &BackupBlockId) -> Result<()> {
        let mut hasher = Sha3_256::new();
        hasher.update(&data);
        let id_bytes = hasher.finalize();
        let id_str = hex::encode(id_bytes);
        let path = self.path.join(&id_str);
        let id_file = std::fs::File::create(path)
            .or_else(|e| error("Could not write block to block index", Some(e.into())))?;
        backup_block_id
            .serialize(&mut Serializer::new(id_file))
            .or_else(|e| {
                error(
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
        let path = self.path.join(id_str);
        if let Ok(block_file) = std::fs::File::open(path) {
            let bid = Deserialize::deserialize(&mut Deserializer::new(block_file))
                .or_else(|e| error("Could not deserialize backup block ID", Some(e.into())))?;
            Ok(Some(bid))
        } else {
            Ok(None)
        }
    }
}

pub fn open<'a>(path: &'a Path) -> Result<impl BlockCache + 'a> {
    Ok(BlockCacheImpl { path: path })
}
