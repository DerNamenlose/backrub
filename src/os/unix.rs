use crate::backup::Meta;
use crate::errors::backrub_error;
use crate::errors::Result;
use crate::os::unix::Meta::UnixFsMeta;
use serde::{Deserialize, Serialize};
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/**
 * basic meta data for files
 */
#[derive(Serialize, Deserialize)]
pub struct UnixFsMetaData {
    /**
     * The user id of the owner of the file. Alternatively used to restore the owner of a file.
     */
    pub uid: u32,
    /**
     * The numeric id of the group of the file. Alternatively used to restore the group of the file.
     */
    pub gid: u32,
    /**
     * The POSIX mode bits (with possibly some extensions)
     */
    pub mode: u32,

    /**
     * the size as present in the backup
     */
    pub size: i64,
}

pub fn get_meta_data(path: &Path) -> Result<Meta> {
    let stat = nix::sys::stat::lstat(path)
        .or_else(|e| backrub_error("Could not stat path", Some(e.into())))?;
    Ok(UnixFsMeta(UnixFsMetaData {
        uid: stat.st_uid,
        gid: stat.st_gid,
        mode: stat.st_mode,
        size: stat.st_size,
    }))
}

pub fn set_meta_data(path: &Path, meta: &Meta) -> Result<()> {
    match meta {
        UnixFsMeta(metadata) => {
            std::fs::set_permissions(path, Permissions::from_mode(metadata.mode))
                .or_else(|e| backrub_error("Could not set permissions", Some(e.into())))?;
            nix::unistd::chown(
                path,
                Some(nix::unistd::Uid::from_raw(metadata.uid)),
                Some(nix::unistd::Gid::from_raw(metadata.gid)),
            )
            .or_else(|e| backrub_error("Could not set file ownership", Some(e.into())))
        }
        _ => backrub_error("Only UNIX meta data supported by this OS adapter", None),
    }
}
