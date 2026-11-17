use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::Result;
use kit_core::catalog::Manifest;
use kit_core::facile::{Backend, InstalledTool};
use kit_core::nix::State;
use kit_core::paths;

use super::backends;
use super::Ctx;

/// What this machine has, per backend.
#[derive(Debug, Default)]
pub struct Machine {
    /// Facile tools whose binaries run from the bin directory.
    pub facile: Vec<InstalledTool>,
    /// User Flatpak apps, or `None` when flatpak is missing.
    pub flathub: Option<Vec<String>>,
    /// The attributes in kit's Nix profile as last recorded, empty when never synced.
    pub nix: Vec<String>,
    /// Whether `nix` is on PATH.
    pub nix_available: bool,
}

/// Looks at the bin directory, the user Flatpak installation and kit's Nix record.
pub fn probe(ctx: &Ctx, backend: &Backend, manifest: &Manifest) -> Result<Machine> {
    let flathub = match backends::flatpak() {
        Some(f) => Some(f.list_installed()?),
        None => None,
    };
    let nix = State::load(&paths::state_dir()?)?
        .filter(|s| !s.stale)
        .map(|s| s.attrs)
        .unwrap_or_default();
    Ok(Machine {
        facile: backend.installed(manifest, &ctx.bin_dir),
        flathub,
        nix,
        nix_available: backends::nix()?.is_some(),
    })
}

/// The latest tag of each installed Facile tool's repo: from the daily cache, or live with `now_live`.
pub fn latest(
    backend: &Backend,
    manifest: &Manifest,
    machine: &Machine,
    live: bool,
) -> HashMap<String, String> {
    let repos: Vec<String> = machine
        .facile
        .iter()
        .filter_map(|i| manifest.get(&i.name).map(|t| t.repo.clone()))
        .collect();
    if live {
        return backend.github.latest(&repos);
    }
    backend
        .latest_cache()
        .resolve(&backend.github, &repos, SystemTime::now())
}
