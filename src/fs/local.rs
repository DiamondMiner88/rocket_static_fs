use super::{Entry, FileSystem};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Implements the FileSystem trait to handle a local directory.
pub struct LocalFileSystem {
    path: PathBuf,
}

impl LocalFileSystem {
    pub fn new<P: AsRef<Path>>(path: P) -> LocalFileSystem {
        LocalFileSystem {
            path: path.as_ref().to_owned(),
        }
    }
}

impl FileSystem for LocalFileSystem {
    type Read = File;

    fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
        self.path.join(path).is_file()
    }

    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        self.path.join(path).is_dir()
    }

    fn last_modified<P: AsRef<Path>>(&self, path: P) -> Result<SystemTime, Box<Error>> {
        let modified = self.path.join(path).metadata()?.modified()?;
        Ok(modified)
    }

    fn size<P: AsRef<Path>>(&self, path: P) -> Result<u64, Box<Error>> {
        let len = self.path.join(path).metadata()?.len();
        Ok(len)
    }

    fn open<P: AsRef<Path>>(
        &self,
        path: P,
        start: Option<u64>,
    ) -> Result<<Self as FileSystem>::Read, Box<Error>> {
        let mut f = File::open(self.path.join(path))?;
        if let Some(start) = start {
            f.seek(SeekFrom::Start(start))?;
        }
        Ok(f)
    }

    fn path_valid<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = self.path.join(path);
        path.starts_with(&self.path)
    }

    fn entries<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Entry>, Box<Error>> {
        let dir = fs::read_dir(self.path.join(path.as_ref()))?;
        let mut entries = Vec::new();
        for f in dir {
            let f = f?;
            let meta = f.metadata()?;
            let filename = f.file_name().to_str().unwrap().to_string();

            if meta.is_file() {
                let size = meta.len();
                let modified = meta.modified()?;
                entries.push(Entry::File(filename, size, modified));
            } else if meta.is_dir() {
                entries.push(Entry::Dir(filename));
            }
            // TODO: Are there other possibilites? How are symlinks noted?
        }
        Ok(entries)
    }
}
