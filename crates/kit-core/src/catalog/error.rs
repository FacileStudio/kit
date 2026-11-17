use std::path::PathBuf;

use super::fetch::FetchError;

/// Everything that can go wrong while reading the catalog or the sources file.
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    /// A file existed but could not be read.
    #[error("reading {path}: {source}")]
    Read {
        /// The file kit tried to read.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },
    /// A catalog, sources file or repo config did not have the expected shape.
    #[error("{0}")]
    Invalid(String),
    /// The parsed catalog had no tools in it.
    #[error("catalog lists no tools")]
    NoTools,
    /// The catalog built into kit is broken, which only a bad release can cause.
    #[error("the embedded catalog is invalid, reinstall kit: {0}")]
    Embedded(String),
    /// A repository has neither a facile.toml nor a `[facile]` block in mise.toml.
    #[error("no facile.toml or [facile] block in mise.toml")]
    NoFacileToml,
    /// The network request failed.
    #[error(transparent)]
    Fetch(#[from] FetchError),
    /// No home directory was found.
    #[error(transparent)]
    Home(#[from] std::io::Error),
}
