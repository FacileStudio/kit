use std::fs;

use super::load::{Loader, REMOTE_URL};
use super::{cache, CatalogError, Manifest, EMBEDDED};

/// The override catalog, when one is set and parses. A stray path falls through instead of failing.
pub(crate) fn local_catalog(loader: &Loader) -> Option<Manifest> {
    let raw = fs::read(loader.local.as_ref()?).ok()?;
    Manifest::parse_bytes(&raw).ok()
}

/// A fresh cache, else the network, else a stale cache, else the embedded copy.
pub(crate) fn remote_or_embedded(loader: &Loader) -> Result<Manifest, CatalogError> {
    let path = loader.cache_path();
    let cached = fs::read(&path)
        .ok()
        .and_then(|raw| Manifest::parse_bytes(&raw).ok());
    if let Some(m) = cached.as_ref().filter(|_| cache::fresh(&path)) {
        return Ok(m.clone());
    }
    if let Some(m) = fetch_remote(loader) {
        return Ok(m);
    }
    match cached {
        Some(m) => Ok(m),
        None => embedded(),
    }
}

/// The catalog compiled into kit.
pub(crate) fn embedded() -> Result<Manifest, CatalogError> {
    Manifest::parse(EMBEDDED).map_err(|e| CatalogError::Embedded(e.to_string()))
}

fn fetch_remote(loader: &Loader) -> Option<Manifest> {
    let raw = loader.fetch.get(REMOTE_URL).ok()?;
    let m = Manifest::parse_bytes(&raw).ok()?;
    cache::write(&loader.cache_path(), &raw);
    Some(m)
}
