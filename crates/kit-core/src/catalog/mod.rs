//! The Facile tool catalog: the embedded copy, the daily refresh from main, and the user's own sources.

mod cache;
mod chain;
mod discover;
mod error;
mod fetch;
mod lists;
mod load;
mod merge;
mod sources;
mod sources_entry;
mod sources_parse;
mod spec;
mod toml_spec;
mod types;

#[cfg(test)]
mod tests;

pub use error::CatalogError;
pub use fetch::{Fetch, FetchError, HttpFetch};
pub use lists::parse_list_source;
pub use load::{catalog_override, Loader, REMOTE_URL};
pub use merge::merge;
pub use sources::{
    parse_sources, sources_path, sources_paths, sources_paths_in, ListSource, SourceConfig,
};
pub use spec::{match_version, SingleSource};
pub use types::{Manifest, Tool};

/// The catalog compiled into kit, the last resort when the network and the cache both fail.
pub const EMBEDDED: &str = include_str!("../../catalog/tools.yml");

/// Finds one tool by name, case-insensitively, in the merged catalog from the default locations.
pub fn lookup(name: &str) -> Result<Option<Tool>, CatalogError> {
    Ok(Loader::from_env()?.load()?.get(name).cloned())
}
