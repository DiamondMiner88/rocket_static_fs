use super::{Entry, FileSystem};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::SystemTime;
use walkdir::WalkDir;

/// Provides a FileSystem which is embedded in the binary.
///
/// # Usage
///
/// First you need to create a package:
///
/// In `build.rs`:
///
/// ```rust,no_run
/// extern crate rocket_static_fs;
///
/// use rocket_static_fs::fs::create_package_from_dir;
/// use std::fs::File;
///
/// fn main() {
///     let package_file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/dummy.pack");
///     let assets_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/assets");
///     let mut package_file = File::create(package_file_path).unwrap();
///     create_package_from_dir(&assets_dir, &mut package_file);
/// }
/// ```
///
/// This will create the package every time you build your application.
///
/// To finally load it in your application.
///
/// In `main.rs`:
///
/// ```rust,no_run
/// extern crate rocket_static_fs;
///
/// use rocket_static_fs::fs::EmbeddedFileSystem;
///
/// fn main() {
///     let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/dummy.pack"));
///     let fs = EmbeddedFileSystem::from_bytes(bytes).unwrap();
///
///     // Do your setup like shown in root of the documentation.
/// }
/// ```
pub struct EmbeddedFileSystem {
    package: Package,
}

impl EmbeddedFileSystem {
    pub fn from_bytes(bytes: &'static [u8]) -> Result<Self, Box<dyn Error>> {
        let package = Package::from_bytes(bytes)?;
        Ok(EmbeddedFileSystem { package })
    }
}

#[rocket::async_trait]
impl FileSystem for EmbeddedFileSystem {
    type Read = Cursor<&'static [u8]>;

    async fn is_file<P>(&self, path: P) -> bool
        where P: AsRef<Path> + Send
    {
        self.package
            .files
            .contains_key(path.as_ref().to_str().unwrap())
    }

    async fn is_dir<P>(&self, path: P) -> bool
        where P: AsRef<Path> + Send
    {
        self.package.is_dir(path)
    }

    async fn last_modified<P>(&self, path: P) -> Result<SystemTime, Box<dyn Error>>
        where P: AsRef<Path> + Send
    {
        match self.package.files.get(path.as_ref().to_str().unwrap()) {
            Some(file) => Ok(file.last_modified.into()),
            None => Err(Box::new(crate::Error::new("file does not exist"))),
        }
    }

    async fn size<P>(&self, path: P) -> Result<u64, Box<dyn Error>>
        where P: AsRef<Path> + Send
    {
        match self.package.files.get(path.as_ref().to_str().unwrap()) {
            Some(file) => Ok(file.len),
            None => Err(Box::new(crate::Error::new("file does not exist"))),
        }
    }

    async fn open<P>(
        &self,
        path: P,
        start: Option<u64>,
    ) -> Result<<Self as FileSystem>::Read, Box<dyn Error>>
        where P: AsRef<Path> + Send
    {
        let mut reader = self.package.open(path)?;
        if let Some(start) = start {
            reader.seek(SeekFrom::Start(start))?;
        }
        Ok(reader)
    }

    async fn path_valid<P>(&self, path: P) -> bool
        where P: AsRef<Path> + Send
    {
        self.package
            .files
            .contains_key(path.as_ref().to_str().unwrap())
    }

    async fn entries<P>(&self, path: P) -> Result<Vec<Entry>, Box<dyn Error>>
        where P: AsRef<Path> + Send
    {
        self.package.entries(path)
    }
}

struct Package {
    files: HashMap<String, InternalFile>,
    data: &'static [u8],
}

struct InternalFile {
    last_modified: DateTime<Utc>,
    len: u64,
    start: u64,
}

impl Package {
    pub fn from_bytes(bytes: &'static [u8]) -> Result<Self, Box<dyn Error>> {
        let mut cursor = Cursor::new(bytes);
        let meta_len = cursor.read_u64::<BigEndian>()?;

        let mut files = HashMap::new();
        let mut read = 0;

        while read < meta_len {
            let cursor_start = cursor.position();
            let path_len = cursor.read_u64::<BigEndian>()? as u64;
            let mut path = String::new();
            let cursor_clone = cursor.clone();
            let mut path_reader = cursor_clone.take(path_len);
            path_reader.read_to_string(&mut path)?;
            cursor.seek(SeekFrom::Current(path_len as i64))?;

            let last_modified_seconds = cursor.read_i64::<BigEndian>()?;
            let last_modified: DateTime<Utc> = Utc.timestamp(last_modified_seconds, 0);

            let len = cursor.read_u64::<BigEndian>()?;
            let start = cursor.read_u64::<BigEndian>()?;

            let cursor_end = cursor.position();

            read += cursor_end - cursor_start;

            files.insert(
                path,
                InternalFile {
                    last_modified,
                    len,
                    start,
                },
            );
        }

        let data = &bytes[(meta_len + 8) as usize..];
        Ok(Package { files, data })
    }

    fn open<P>(&self, path: P) -> Result<Cursor<&'static [u8]>, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        match self.files.get(path.as_ref().to_str().unwrap()) {
            Some(file) => {
                let start = file.start as usize;
                let end = (file.start + file.len) as usize;
                let slice = &self.data[start..end];
                Ok(Cursor::new(slice))
            }
            None => Err(Box::new(crate::Error::new("file does not exist"))),
        }
    }

    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        // / is always a dir
        if let Some("/") = path.as_ref().to_str() {
            return true;
        }

        let mut path_str = path.as_ref().to_str().unwrap().to_string();

        // The path most likely starts with a / but our package paths do not
        // so we remove it here if it exists
        if path_str.starts_with('/') {
            path_str = path_str.replacen('/', "", 1);
        }

        for (k, _v) in self.files.iter() {
            // Skip every file which doesn't match our path
            if !k.starts_with(&path_str) {
                continue;
            }

            // If the right starts with a slash, it is a directory
            let right = k.replacen(&path_str, "", 1);
            if right.starts_with('/') {
                return true;
            }
        }

        false
    }

    fn entries<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Entry>, Box<dyn Error>> {
        let mut path_str = path.as_ref().to_str().unwrap().to_string();

        // The path most likely starts with a / but our package paths do not
        // so we remove it here if it exists
        if path_str.starts_with('/') {
            path_str = path_str.replacen('/', "", 1);
        }

        let mut entries = Vec::new();
        for (k, v) in self.files.iter() {
            // Skip every file which doesn't match our path
            if !k.starts_with(&path_str) {
                continue;
            }

            // If the right side starts with a slash we are pretty much in the directory
            let right = k.replacen(&path_str, "", 1);
            if right.starts_with('/') {
                let right = right.replacen('/', "", 1);
                // If the right side still contains a slash, we still have sub-directories
                if right.contains('/') {
                    let dir_name = right.splitn(2, '/').collect::<Vec<&str>>()[0];
                    entries.push(Entry::Dir(dir_name.to_string()));
                } else {
                    // This can't possibly go wrong
                    entries.push(Entry::File(
                        right.to_string(),
                        v.len,
                        v.last_modified.into(),
                    ));
                }
            }
        }

        entries.sort_by_key(|s| match *s {
            Entry::File(ref name, _, _) => name.to_string(),
            Entry::Dir(ref name) => name.to_string(),
        });

        Ok(entries)
    }
}

/// Writes a package to the given writer. The paths will be as given in `input_files`.
/// The path to read the files will be joined starting at the `root` path.
///
/// Most likely you want to use `create_package_from_dir` instead.
pub fn write_package<W, T, P>(root: P, input_files: &[T], writer: &mut W) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
    W: Write + WriteBytesExt,
    T: AsRef<str> + Clone + Ord,
{
    let mut files = Vec::from(input_files);
    files.sort();

    let mut file_sizes = Vec::new();
    let mut file_modification_times = Vec::new();
    let mut meta_len = 0;
    for f in &files {
        // 8 * 4 = 32 cause of last_modified + path_len + start + len which are all 64bit
        meta_len += 32;
        meta_len += f.as_ref().as_bytes().len();

        let meta = root.as_ref().join(f.as_ref()).metadata()?;
        let file_size = meta.len();
        file_sizes.push(file_size);

        let mod_time = meta.modified()?;
        file_modification_times.push(mod_time);
    }

    let mut data_offset = 0;
    writer.write_u64::<BigEndian>(meta_len as u64)?;

    for (i, f) in files.iter().enumerate() {
        // written in the following order: path_len, path, last_modified, len, start
        writer.write_u64::<BigEndian>(f.as_ref().as_bytes().len() as u64)?;
        write!(writer, "{}", f.as_ref().replace('\\', "/"))?;

        let last_modified: DateTime<Utc> = DateTime::from(file_modification_times[i]);
        writer.write_i64::<BigEndian>(last_modified.timestamp())?;

        let file_size = &file_sizes[i];
        writer.write_u64::<BigEndian>(*file_size)?;

        writer.write_u64::<BigEndian>(data_offset as u64)?;

        data_offset += (*file_size) as usize;
    }

    for f in &files {
        let mut file = File::open(root.as_ref().join(f.as_ref()))?;
        io::copy(&mut file, writer)?;
    }

    Ok(())
}

/// Creates a package from the given dir to the provided writer.
///
/// The file paths in the resulting package will start relative to `dir`.
///
/// # Example
///
/// ```rust,no_run
/// use std::fs::File;
/// use rocket_static_fs::fs::create_package_from_dir;
///
/// fn main() {
///     let mut f = File::create("assets.pack").unwrap();
///     create_package_from_dir("assets", &mut f).unwrap();
/// }
/// ```
pub fn create_package_from_dir<P, W>(dir: P, writer: &mut W) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
    W: Write,
{
    let root = dir.as_ref().canonicalize()?;
    let mut files = Vec::new();
    for entry in WalkDir::new(&dir) {
        let entry = entry?;
        if entry.metadata()?.is_file() {
            let file_path = entry.path().canonicalize()?;
            let path = file_path
                .to_str()
                .unwrap()
                .replacen(root.to_str().unwrap(), "", 1);

            files.push(
                path.trim_start_matches('/')
                    .trim_start_matches('\\')
                    .to_string(),
            )
        }
    }

    write_package(root, &files, writer)
}

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use super::*;
    #[allow(unused)]
    use std::fs::File;

    #[test]
    fn test_create_package_from_dir_and_read_back() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/assets");
        let package_path = concat!(env!("CARGO_MANIFEST_DIR"), "/target/test.package");
        let mut file = File::create(package_path).unwrap();
        create_package_from_dir(dir, &mut file).expect("unable to create package");

        let package = Package::from_bytes(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/test.package"
        )));

        match package {
            Ok(p) => {
                assert_eq!(p.files.len(), 5);
                assert!(p.files.get("hello.txt").is_some());
                assert!(p.files.get("inner/other.txt").is_some());

                let hello_world = p.files.get("hello.txt").unwrap();
                assert_eq!(hello_world.len, "Hello World!".as_bytes().len() as u64);
                let mut hello_str = String::new();
                p.open("hello.txt")
                    .unwrap()
                    .read_to_string(&mut hello_str)
                    .unwrap();
                assert_eq!(hello_str, "Hello World!");

                assert!(p.is_dir("/"));
                assert!(p.is_dir("/inner"));
                assert!(!p.is_dir("/not-there"));
                assert!(!p.is_dir("/hello.txt"));
                assert!(!p.is_dir("/inner/other.txt"));

                let entries = p.entries("/inner").unwrap();
                assert_eq!(entries.len(), 2);
                match entries[0] {
                    Entry::Dir(ref name) => assert_eq!(name.as_str(), "deeper"),
                    _ => panic!("entry is not a dir"),
                }

                match entries[1] {
                    Entry::File(ref name, _, _) => assert_eq!(name.as_str(), "other.txt"),
                    _ => panic!("entry is not a file"),
                };
            }
            Err(e) => panic!(format!(
                "unable to read test.package, maybe you just need to re-run the test: {}",
                e
            )),
        }
    }
}
