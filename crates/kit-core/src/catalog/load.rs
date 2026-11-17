use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use super::fetch::{Fetch, HttpFetch};
use super::{cache, chain, merge, sources, CatalogError, Manifest, SourceConfig};
use crate::paths;

/// Where kit refreshes the catalog from.
pub const REMOTE_URL: &str =
    "https://raw.githubusercontent.com/FacileStudio/kit/main/crates/kit-core/catalog/tools.yml";

/// Reads the catalog. Every input is a field, so tests run offline in a temporary directory.
pub struct Loader {
    /// Holds `tools.yml` and the `lists/` cache.
    pub cache_dir: PathBuf,
    /// Candidate sources files, first existing one wins.
    pub sources: Vec<PathBuf>,
    /// A local catalog that replaces everything, from `KIT_CATALOG` or `FACILE_CATALOG`.
    pub local: Option<PathBuf>,
    /// The network.
    pub fetch: Box<dyn Fetch>,
}

/// The local catalog override: `KIT_CATALOG`, then the legacy `FACILE_CATALOG`. Empty values do not count.
pub fn catalog_override(kit: Option<OsString>, facile: Option<OsString>) -> Option<PathBuf> {
    [kit, facile]
        .into_iter()
        .flatten()
        .find(|v| !v.is_empty())
        .map(PathBuf::from)
}

impl Loader {
    /// The real locations: the kit cache directory, the sources files, the env override and HTTPS.
    pub fn from_env() -> Result<Loader, CatalogError> {
        Ok(Loader {
            cache_dir: paths::cache_dir()?,
            sources: sources::sources_paths(),
            local: catalog_override(
                std::env::var_os("KIT_CATALOG"),
                std::env::var_os("FACILE_CATALOG"),
            ),
            fetch: Box::new(HttpFetch::new(Duration::from_secs(5))),
        })
    }

    /// The catalog with user sources merged on top. A local override is returned alone.
    pub fn load(&self) -> Result<Manifest, CatalogError> {
        if let Some(m) = chain::local_catalog(self) {
            return Ok(m);
        }
        Ok(merge::merge_sources(self, chain::remote_or_embedded(self)?))
    }

    /// The catalog without user sources, to compare them against.
    pub fn load_base(&self) -> Result<Manifest, CatalogError> {
        if let Some(m) = chain::local_catalog(self) {
            return Ok(m);
        }
        chain::remote_or_embedded(self)
    }

    /// Fetches the remote catalog now, updates the cache, and merges user sources.
    pub fn refresh(&self) -> Result<Manifest, CatalogError> {
        let raw = self.fetch.get(REMOTE_URL)?;
        let m = Manifest::parse_bytes(&raw)?;
        cache::write(&self.cache_path(), &raw);
        Ok(merge::merge_sources(self, m))
    }

    /// Reads the first sources file that exists. No file means an empty config.
    pub fn load_sources(&self) -> Result<SourceConfig, CatalogError> {
        sources::load_from(&self.sources)
    }

    /// The cached copy of the remote catalog.
    pub fn cache_path(&self) -> PathBuf {
        self.cache_dir.join("tools.yml")
    }
}
