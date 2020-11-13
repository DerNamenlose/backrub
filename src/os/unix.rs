use crate::backup::Meta;
use crate::errors::backrub_error;
use crate::errors::Error;
use crate::errors::Result;
use crate::os::unix::Meta::UnixMeta;
use nix::sys::stat::SFlag;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/**
 * basic meta-data supported by most Unix objects
 */
#[derive(Serialize, Deserialize)]
pub struct UnixCommonMeta {
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
}

/**
 * basic meta data for files
 */
#[derive(Serialize, Deserialize)]
pub struct UnixFileMetaData {
    /**
     * Common meta data
     */
    pub common: UnixCommonMeta,

    /**
     * the size as present in the backup
     */
    pub size: i64,
}

/**
 * basic meta data for files
 */
#[derive(Serialize, Deserialize)]
pub struct UnixSymlinkMetaData {
    /**
     * Target the symlink points to
     */
    pub target: String,
}

#[derive(Serialize, Deserialize)]
pub enum UnixFsMeta {
    File(UnixFileMetaData),
    Dir(UnixCommonMeta),
    Symlink(UnixSymlinkMetaData),
}

impl Display for UnixFsMeta {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        match self {
            UnixFsMeta::File(fmeta) => write!(
                formatter,
                "File(size={} bytes, permissions={:o})",
                fmeta.size, fmeta.common.mode
            ),
            UnixFsMeta::Dir(permissions) => {
                write!(formatter, "Dir(permissions={:o})", permissions.mode)
            }
            UnixFsMeta::Symlink(lmeta) => write!(formatter, "Link({})", lmeta.target),
        }
    }
}

pub fn get_meta_data(path: &Path) -> Result<Meta> {
    log::trace!("Retrieving meta data for {}", path.display());
    let stat = nix::sys::stat::lstat(path)
        .or_else(|e| backrub_error("Could not stat path", Some(e.into())))?;
    let sflags = SFlag::from_bits_truncate(stat.st_mode);
    if sflags.contains(SFlag::S_IFREG) {
        Ok(UnixMeta(UnixFsMeta::File(UnixFileMetaData {
            common: UnixCommonMeta {
                uid: stat.st_uid,
                gid: stat.st_gid,
                mode: stat.st_mode,
            },
            size: stat.st_size,
        })))
    } else if sflags.contains(SFlag::S_IFDIR) {
        Ok(UnixMeta(UnixFsMeta::Dir(UnixCommonMeta {
            uid: stat.st_uid,
            gid: stat.st_gid,
            mode: stat.st_mode,
        })))
    } else if sflags.contains(SFlag::S_IFLNK) {
        let target = std::fs::read_link(path)
            .or_else(|e| backrub_error("Could not resolve symlink", Some(e.into())))?;
        let target_str = target.to_str().ok_or(Error {
            message: "Could not decode link target to the ",
            cause: None,
        })?;
        Ok(UnixMeta(UnixFsMeta::Symlink(UnixSymlinkMetaData {
            target: String::from(target_str),
        })))
    } else {
        backrub_error("Unsupported object type", None)
    }
}

pub fn set_meta_data(path: &Path, meta: &Meta) -> Result<()> {
    log::trace!("Setting meta data for {}", path.display());
    log::trace!("Meta data is: {}", meta);
    match meta {
        UnixMeta(UnixFsMeta::File(metadata)) => set_file_metadata(path, &metadata),
        UnixMeta(UnixFsMeta::Dir(metadata)) => set_common_meta(path, &metadata),
        UnixMeta(UnixFsMeta::Symlink(_)) => Ok(()),
    }
}

fn set_file_metadata(path: &Path, meta: &UnixFileMetaData) -> Result<()> {
    set_common_meta(path, &meta.common)
}

fn set_common_meta(path: &Path, meta: &UnixCommonMeta) -> Result<()> {
    std::fs::set_permissions(path, Permissions::from_mode(meta.mode))
        .or_else(|e| backrub_error("Could not set permissions", Some(e.into())))?;
    nix::unistd::chown(
        path,
        Some(nix::unistd::Uid::from_raw(meta.uid)),
        Some(nix::unistd::Gid::from_raw(meta.gid)),
    )
    .or_else(|e| backrub_error("Could not set file ownership", Some(e.into())))
}
