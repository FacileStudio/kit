//! Locating the list file and changing it safely: exclusive lock, then atomic replace.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use fd_lock::RwLock;

use super::{parse, KitFile, ListError};
use crate::paths;

/// The list file path: `explicit` (from `--file`), then `KIT_FILE`, then `<config dir>/kit.yml`.
pub fn resolve_path(explicit: Option<&Path>) -> std::io::Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
    }
    match std::env::var_os("KIT_FILE") {
        Some(path) if !path.is_empty() => Ok(PathBuf::from(path)),
        _ => Ok(paths::config_dir()?.join("kit.yml")),
    }
}

/// A list file on disk, with the seed it starts from when missing.
#[derive(Debug, Clone)]
pub struct ListStore {
    path: PathBuf,
    seed: PathBuf,
}

impl ListStore {
    /// A store for `path`, seeded from [`paths::system_seed`].
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            seed: paths::system_seed(),
        }
    }

    /// Replaces the seed path, for tests and packagers.
    pub fn with_seed(mut self, seed: impl Into<PathBuf>) -> Self {
        self.seed = seed.into();
        self
    }

    /// The list file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Reads the list without locking. A missing file yields the seed, or an empty list.
    pub fn read(&self) -> Result<KitFile, ListError> {
        parse::load_or_seed(&self.path, &self.seed)
    }

    /// Locks the list, runs `change` on it and replaces the file atomically. Blocks while another edit holds the lock.
    pub fn edit<T>(&self, change: impl FnOnce(&mut KitFile) -> T) -> Result<T, ListError> {
        let io = |source| ListError::Io {
            path: self.path.clone(),
            source,
        };
        let mut lock = RwLock::new(self.open_lock().map_err(io)?);
        let _guard = lock.write().map_err(io)?;
        let mut file = self.read()?;
        let result = change(&mut file);
        let text = parse::render(&file, &self.path)?;
        self.write_atomic(&text).map_err(io)?;
        Ok(result)
    }

    fn open_lock(&self) -> std::io::Result<File> {
        if let Some(dir) = self.path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
            fs::create_dir_all(dir)?;
        }
        let mut name = self.path.file_name().unwrap_or_default().to_os_string();
        name.push(".lock");
        OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(self.path.with_file_name(name))
    }

    fn write_atomic(&self, text: &str) -> std::io::Result<()> {
        let dir = self
            .path
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut temp = tempfile::NamedTempFile::new_in(dir)?;
        temp.write_all(text.as_bytes())?;
        temp.as_file().sync_all()?;
        temp.persist(&self.path).map_err(|err| err.error)?;
        Ok(())
    }
}
