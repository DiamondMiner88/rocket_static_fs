//! Includes the FileSystem trait and built-in implementations.

use chrono::prelude::*;
use std::error::Error;
use std::io::Read;
use std::path::Path;
use std::time::SystemTime;

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
                    .format(::LAST_MODIFIED_DATE_FORMAT)
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
pub trait FileSystem {
    type Read: Read;
    fn is_file<P: AsRef<Path>>(&self, path: P) -> bool;
    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool;
    fn last_modified<P: AsRef<Path>>(&self, path: P) -> Result<SystemTime, Box<Error>>;
    fn size<P: AsRef<Path>>(&self, path: P) -> Result<u64, Box<Error>>;
    fn open<P: AsRef<Path>>(
        &self,
        path: P,
        start: Option<u64>,
    ) -> Result<<Self as FileSystem>::Read, Box<Error>>;
    fn path_valid<P: AsRef<Path>>(&self, path: P) -> bool;
    fn entries<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Entry>, Box<Error>>;
}
