use super::{Entry, FileSystem};
use std::error::Error;
use std::fs;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs::File;
use tokio::io::AsyncSeekExt;

/// Implements the FileSystem trait to handle a local directory.
pub struct LocalFileSystem {
    path: PathBuf,
}

impl LocalFileSystem
{
    pub fn new<P>(path: P) -> LocalFileSystem
        where P: AsRef<Path> + Send
    {
        LocalFileSystem {
            path: path.as_ref().to_owned(),
        }
    }
}

#[rocket::async_trait]
impl FileSystem for LocalFileSystem {
    type Read = File;

    async fn is_file<P>(&self, path: P) -> bool
        where P: AsRef<Path> + Send
    {
        self.path.join(path).is_file()
    }

    async fn is_dir<P>(&self, path: P) -> bool
        where P: AsRef<Path> + Send
    {
        self.path.join(path).is_dir()
    }

    async fn last_modified<P>(&self, path: P) -> Result<SystemTime, Box<dyn Error>>
        where P: AsRef<Path> + Send
    {
        let modified = self.path.join(path).metadata()?.modified()?;
        Ok(modified)
    }

    async fn size<P>(&self, path: P) -> Result<u64, Box<dyn Error>>
        where P: AsRef<Path> + Send
    {
        let len = self.path.join(path).metadata()?.len();
        Ok(len)
    }

    async fn open<P>(
        &self,
        path: P,
        start: Option<u64>,
    ) -> Result<<Self as FileSystem>::Read, Box<dyn Error>>
        where P: AsRef<Path> + Send
    {
        let mut f = File::open(self.path.join(path)).await?;
        if let Some(start) = start {
            f.seek(SeekFrom::Start(start)).await?;
        }
        Ok(f)
    }

    async fn path_valid<P>(&self, path: P) -> bool
        where P: AsRef<Path> + Send
    {
        let path = self.path.join(path);
        path.starts_with(&self.path)
    }

    async fn entries<P>(&self, path: P) -> Result<Vec<Entry>, Box<dyn Error>>
        where P: AsRef<Path> + Send
    {
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
            // TODO: Are there other possibilities? How are symlinks noted?
        }
        Ok(entries)
    }
}
