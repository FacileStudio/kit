//! The list file: what the user wants installed, one string list per backend.

mod parse;
mod source;
mod store;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

use serde::Serialize;

pub use source::Source;
pub use store::{resolve_path, ListStore};

/// The only list file version kit reads and writes.
pub const VERSION: u32 = 1;

/// Everything that can go wrong reading, writing or naming list entries.
#[derive(Debug, thiserror::Error)]
pub enum ListError {
    /// A file operation on `path` failed.
    #[error("{path}: {source}")]
    Io {
        /// The file kit was touching.
        path: PathBuf,
        /// The underlying error.
        source: std::io::Error,
    },
    /// The file is not valid YAML or a section is not a list of strings.
    #[error("{path}: {source}")]
    Yaml {
        /// The file kit was reading.
        path: PathBuf,
        /// The underlying error.
        source: serde_yaml::Error,
    },
    /// The top level is not a mapping.
    #[error("{path}: expected a mapping with version, facile, nix and flathub")]
    NotMapping {
        /// The file kit was reading.
        path: PathBuf,
    },
    /// A top-level key kit does not know.
    #[error("{path}: unknown key `{key}` (expected version, facile, nix or flathub)")]
    UnknownKey {
        /// The file kit was reading.
        path: PathBuf,
        /// The offending key.
        key: String,
    },
    /// A version other than [`VERSION`].
    #[error("{path}: unsupported version {found} (kit reads version {VERSION})")]
    Version {
        /// The file kit was reading.
        path: PathBuf,
        /// The version the file declares.
        found: u64,
    },
    /// A `backend:` prefix that is not facile, nix or flathub.
    #[error("unknown source `{0}` (expected facile, nix or flathub)")]
    UnknownSource(String),
}

/// The parsed list file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KitFile {
    /// Always [`VERSION`].
    pub version: u32,
    /// Facile tools, by catalog name.
    pub facile: Vec<String>,
    /// Nix attributes.
    pub nix: Vec<String>,
    /// Flathub application IDs.
    pub flathub: Vec<String>,
}

impl Default for KitFile {
    fn default() -> Self {
        Self {
            version: VERSION,
            facile: Vec::new(),
            nix: Vec::new(),
            flathub: Vec::new(),
        }
    }
}

impl KitFile {
    /// The entries of one section, in file order.
    pub fn entries(&self, source: Source) -> &[String] {
        match source {
            Source::Facile => &self.facile,
            Source::Nix => &self.nix,
            Source::Flathub => &self.flathub,
        }
    }

    /// Appends `name` to `source` unless it is already there. Returns whether the file changed.
    pub fn add(&mut self, source: Source, name: &str) -> bool {
        let list = self.section_mut(source);
        if list.iter().any(|entry| entry == name) {
            return false;
        }
        list.push(name.to_string());
        true
    }

    /// Removes `name` from `source`, keeping the order of the rest. Returns whether the file changed.
    pub fn remove(&mut self, source: Source, name: &str) -> bool {
        let list = self.section_mut(source);
        let before = list.len();
        list.retain(|entry| entry != name);
        list.len() != before
    }

    /// Reads the list file at `path`, seeding from the system list when it is missing.
    pub fn read(path: impl Into<PathBuf>) -> Result<Self, ListError> {
        ListStore::new(path).read()
    }

    /// Runs `change` on the list file at `path` under an exclusive lock, then writes it atomically.
    pub fn edit<T>(
        path: impl Into<PathBuf>,
        change: impl FnOnce(&mut Self) -> T,
    ) -> Result<T, ListError> {
        ListStore::new(path).edit(change)
    }

    fn section_mut(&mut self, source: Source) -> &mut Vec<String> {
        match source {
            Source::Facile => &mut self.facile,
            Source::Nix => &mut self.nix,
            Source::Flathub => &mut self.flathub,
        }
    }
}
