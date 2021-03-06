use crate::errors::error;
use std::fs::File;
use std::io::Read;
use walkdir::DirEntry;
use walkdir::WalkDir;

pub struct FsSource<'a, F>
where
    F: Fn(&walkdir::DirEntry) -> bool,
{
    path: String,
    filter: &'a F,
}

impl<F> FsSource<'_, F>
where
    F: Fn(&walkdir::DirEntry) -> bool,
{
    pub fn new<'a>(path: &str, filter: &'a F) -> FsSource<'a, F>
    where
        F: Fn(&walkdir::DirEntry) -> bool,
    {
        FsSource {
            path: String::from(path),
            filter: filter,
        }
    }

    pub fn objects(&self) -> FsObjectIterator {
        FsObjectIterator {
            current: Box::new(
                WalkDir::new(&self.path)
                    .into_iter()
                    .filter_entry(&(*self.filter))
                    .filter_map(|e| e.ok()),
            ),
        }
    }

    pub fn open_entry(&self, path: &str) -> crate::errors::Result<FsBlockSource> {
        Ok(FsBlockSource {
            file: File::open(&path).or_else(|e| error("Could not open entry", Some(e.into())))?,
        })
    }
}

pub struct FsObjectIterator<'a> {
    current: Box<dyn Iterator<Item = DirEntry> + 'a>,
}

impl Iterator for FsObjectIterator<'_> {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.next()
    }
}

pub struct FsBlockSource {
    file: File,
}

impl Iterator for FsBlockSource {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = [0; 1048576];
        let result = self.file.read(&mut buf);
        match result {
            Ok(bytes) => {
                if bytes > 0 {
                    Some(buf[..bytes].to_vec())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
