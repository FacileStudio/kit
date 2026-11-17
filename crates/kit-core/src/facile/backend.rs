use std::ffi::OsString;
use std::path::PathBuf;

use super::github::Github;
use super::latest_cache::LatestCache;
use crate::paths;

/// The machine kit works on. Tests build one by hand so they never read the real home or PATH.
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// The home directory, where agent skills are written.
    pub home: PathBuf,
    /// `XDG_DATA_HOME` when set and not empty.
    pub data_home: Option<PathBuf>,
    /// The PATH used to find commands and agents.
    pub path: OsString,
    /// kit's cache directory, which holds `latest.json`.
    pub cache_dir: PathBuf,
}

impl Env {
    /// The real home, data directory, PATH and cache directory.
    pub fn from_env() -> std::io::Result<Env> {
        Ok(Env {
            home: paths::home()?,
            data_home: std::env::var_os("XDG_DATA_HOME")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from),
            path: std::env::var_os("PATH").unwrap_or_default(),
            cache_dir: paths::cache_dir()?,
        })
    }
}

/// The Facile tool backend: GitHub releases, source builds, agent skills and the running machine.
pub struct Backend {
    /// The release host.
    pub github: Github,
    /// The machine.
    pub env: Env,
}

impl Backend {
    /// The backend on the real network and machine.
    pub fn from_env() -> std::io::Result<Backend> {
        Ok(Backend {
            github: Github::default(),
            env: Env::from_env()?,
        })
    }

    /// The daily cache of latest tags.
    pub fn latest_cache(&self) -> LatestCache {
        LatestCache::in_dir(self.env.cache_dir.clone())
    }
}
