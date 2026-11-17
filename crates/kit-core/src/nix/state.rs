use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::NixError;

/// Where each generation records what it holds, inside the built environment.
pub const ENV_RECORD: &str = "share/kit/env.json";

/// What the last sync built: the nixpkgs revision and the sorted attribute list.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// The nixpkgs revision the profile was built from.
    pub rev: String,
    /// The attributes in the profile, sorted and deduplicated.
    pub attrs: Vec<String>,
    /// True when kit no longer knows what the profile holds, so the next check resyncs.
    #[serde(default)]
    pub stale: bool,
}

impl State {
    /// A record that forces the next `needs_resync` to say yes.
    pub fn stale() -> Self {
        Self {
            stale: true,
            ..Self::default()
        }
    }

    /// `<state_dir>/nix.json`.
    pub fn path(state_dir: &Path) -> PathBuf {
        state_dir.join("nix.json")
    }

    /// The recorded state, or `None` when nothing was synced yet.
    pub fn load(state_dir: &Path) -> Result<Option<Self>, NixError> {
        match fs::read(Self::path(state_dir)) {
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Writes the record atomically to `<state_dir>/nix.json`.
    pub fn save(&self, state_dir: &Path) -> Result<(), NixError> {
        fs::create_dir_all(state_dir)?;
        let tmp = state_dir.join("nix.json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        fs::rename(&tmp, Self::path(state_dir))?;
        Ok(())
    }

    /// The record the current profile generation carries, when it has a readable one.
    pub fn from_profile(profile: &Path) -> Option<Self> {
        let bytes = fs::read(profile.join(ENV_RECORD)).ok()?;
        serde_json::from_slice::<Self>(&bytes)
            .ok()
            .map(|s| Self { stale: false, ..s })
    }
}
