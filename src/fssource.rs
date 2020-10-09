use std::fs::File;
use std::io::Read;
use walkdir::DirEntry;
use walkdir::WalkDir;

pub struct FsSource {
    path: String,
}

impl FsSource {
    pub fn new(path: &str) -> Self {
        FsSource {
            path: String::from(path),
        }
    }

    pub fn objects(&self) -> FsObjectIterator {
        FsObjectIterator {
            current: Box::new(
                WalkDir::new(&self.path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|f| f.file_type().is_file()),
            ),
        }
    }

    pub fn open_entry(&self, path: &str) -> std::io::Result<FsBlockSource> {
        Ok(FsBlockSource {
            file: File::open(&path)?,
        })
    }
}

pub struct FsObjectIterator {
    current: Box<dyn Iterator<Item = DirEntry>>,
}

impl Iterator for FsObjectIterator {
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
        let mut buf = [0; 65536];
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
