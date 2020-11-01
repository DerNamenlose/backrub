use crate::errors::backrub_error;
use crate::errors::Result;
use sha3::{Digest, Sha3_256};

pub trait BlockCache {
    fn add_block(&self, data: &Vec<u8>) -> Result<()>;

    fn has_block(&self, data: &Vec<u8>) -> Result<bool>;
}

struct BlockCacheImpl<'a> {
    path: &'a str,
}

impl BlockCache for BlockCacheImpl<'_> {
    fn add_block(&self, data: &Vec<u8>) -> Result<()> {
        let mut hasher = Sha3_256::new();
        hasher.update(&data);
        let id_bytes = hasher.finalize();
        let id_str = hex::encode(id_bytes);
        let path: std::path::PathBuf = [self.path, &id_str].iter().collect();
        std::fs::File::create(path)
            .or_else(|e| backrub_error("Could not write block to block index", Some(e.into())))?;
        Ok(())
    }

    fn has_block(&self, data: &Vec<u8>) -> Result<bool> {
        let mut hasher = Sha3_256::new();
        hasher.update(&data);
        let id_bytes = hasher.finalize();
        let id_str = hex::encode(id_bytes);
        let path: std::path::PathBuf = [self.path, &id_str].iter().collect();
        Ok(path.exists())
    }
}

pub fn open<'a>(path: &'a str) -> Result<impl BlockCache + 'a> {
    Ok(BlockCacheImpl { path: path })
}
