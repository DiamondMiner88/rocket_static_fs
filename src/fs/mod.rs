//! Includes the FileSystem trait and built-in implementations.

use chrono::prelude::*;
use std::error::Error;
use std::path::Path;
use std::time::SystemTime;
use rocket::tokio::io::AsyncRead;

mod embedded;
mod local;

pub use self::embedded::create_package_from_dir;
pub use self::embedded::write_package;
pub use self::embedded::EmbeddedFileSystem;
pub use self::local::LocalFileSystem;

pub enum Entry {
    File(String, u64, SystemTime),
    Dir(String),
}

#[derive(Serialize)]
pub struct TemplateEntry {
    name: String,
    size: u64,
    last_modified: String,
    is_file: bool,
}

impl<'a> From<&'a Entry> for TemplateEntry {
    fn from(e: &'a Entry) -> Self {
        match e.clone() {
            Entry::File(name, size, last_modified) => {
                let last_modified: DateTime<Utc> = DateTime::from(*last_modified);
                let last_modified = last_modified
                    .format(crate::LAST_MODIFIED_DATE_FORMAT)
                    .to_string();
                TemplateEntry {
                    name: name.to_string(),
                    size: *size,
                    last_modified,
                    is_file: true,
                }
            }
            Entry::Dir(name) => TemplateEntry {
                name: name.to_string(),
                size: 0,
                last_modified: String::new(),
                is_file: false,
            },
        }
    }
}

/// Implement this trait to provide a filesystem to serve from.
#[rocket::async_trait]
pub trait FileSystem {
    type Read: AsyncRead + Send + Unpin;

    async fn is_file<P>(&self, path: P) -> bool
        where P: AsRef<Path> + Send;
    async fn is_dir<P>(&self, path: P) -> bool
        where P: AsRef<Path> + Send;
    async fn last_modified<P>(&self, path: P) -> Result<SystemTime, Box<dyn Error>>
        where P: AsRef<Path> + Send;
    async fn size<P>(&self, path: P) -> Result<u64, Box<dyn Error>>
        where P: AsRef<Path> + Send;
    async fn open<P>(
        &self,
        path: P,
        start: Option<u64>,
    ) -> Result<<Self as FileSystem>::Read, Box<dyn Error>>
        where P: AsRef<Path> + Send;
    async fn path_valid<P>(&self, path: P) -> bool
        where P: AsRef<Path> + Send;
    async fn entries<P>(&self, path: P) -> Result<Vec<Entry>, Box<dyn Error>>
        where P: AsRef<Path> + Send;
}
