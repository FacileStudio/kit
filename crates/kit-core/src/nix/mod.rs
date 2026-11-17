//! Nix backend: one per-user `buildEnv` profile, driven by the `nix` and `nix-env` CLIs.
//!
//! kit never calls `nix profile` (experimental) and never calls sudo.

mod attr;
mod error;
mod run;
mod state;

#[cfg(test)]
mod attr_tests;
#[cfg(test)]
mod cli_tests;
#[cfg(test)]
mod stub;
#[cfg(test)]
mod sync_tests;

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

pub use attr::validate_attr;
pub use error::NixError;
pub use state::State;

use crate::paths;

/// A handle on the kit Nix profile and the binaries that manage it.
#[derive(Debug, Clone)]
pub struct Nix {
    nix: PathBuf,
    nix_env: PathBuf,
    state_dir: PathBuf,
}

impl Nix {
    /// The default handle: `nix` and `nix-env` from PATH, state under [`paths::state_dir`].
    pub fn new() -> Result<Self, NixError> {
        Ok(Self::with_binaries("nix", "nix-env", paths::state_dir()?))
    }

    /// A handle with explicit binaries and state directory, for tests and custom installs.
    pub fn with_binaries(
        nix: impl Into<PathBuf>,
        nix_env: impl Into<PathBuf>,
        state_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            nix: nix.into(),
            nix_env: nix_env.into(),
            state_dir: state_dir.into(),
        }
    }

    /// The profile symlink: `<state_dir>/nix-profile`.
    pub fn profile(&self) -> PathBuf {
        self.state_dir.join("nix-profile")
    }

    /// The revision the `nixpkgs` registry entry resolves to right now.
    pub fn registry_rev(&self) -> Result<String, NixError> {
        let args = run::flagged(["flake", "metadata", "nixpkgs", "--json"]);
        attr::parse_rev(&run::run(&self.nix, "nix flake metadata", args)?)
    }

    /// Builds one `buildEnv` of `attrs` into the profile, prunes old generations and records the result.
    pub fn sync(&self, attrs: &[String]) -> Result<State, NixError> {
        let attrs = attr::normalize(attrs)?;
        let rev = self.registry_rev()?;
        let expr = attr::env_expr(&rev, &attrs)?;
        fs::create_dir_all(&self.state_dir)?;
        let p = self.profile().into_os_string();
        let build = ["build", "--impure", "--no-link", "--profile"].map(OsString::from);
        let tail = [p.clone(), "--expr".into(), expr.into()];
        run::run(
            &self.nix,
            "nix build",
            run::flagged(build.into_iter().chain(tail)),
        )?;
        let prune = [
            "-p".into(),
            p,
            "--delete-generations".into(),
            OsString::from("+10"),
        ];
        run::run(&self.nix_env, "nix-env --delete-generations", prune)?;
        let recorded = State {
            rev,
            attrs,
            stale: false,
        };
        recorded.save(&self.state_dir)?;
        Ok(recorded)
    }

    /// True when the registry revision or the wanted attributes differ from the last sync.
    pub fn needs_resync(&self, attrs: &[String]) -> Result<bool, NixError> {
        let want = attr::normalize(attrs)?;
        match State::load(&self.state_dir)? {
            Some(s) if !s.stale && s.attrs == want => Ok(s.rev != self.registry_rev()?),
            _ => Ok(true),
        }
    }

    /// Rolls the profile back one generation and rewrites the record from it, or marks the record stale.
    pub fn undo(&self) -> Result<State, NixError> {
        let profile = self.profile();
        let args = [
            OsString::from("-p"),
            profile.clone().into(),
            "--rollback".into(),
        ];
        run::run(&self.nix_env, "nix-env --rollback", args)?;
        let recorded = State::from_profile(&profile).unwrap_or_else(State::stale);
        recorded.save(&self.state_dir)?;
        Ok(recorded)
    }

    /// Whether `attr` names a package in the registry nixpkgs.
    pub fn exists(&self, attr: &str) -> Result<bool, NixError> {
        let expr = attr::exists_expr(attr)?;
        let args = run::flagged(["eval", "--impure", "--expr", &expr]);
        match run::run(&self.nix, "nix eval", args)?.trim() {
            "true" => Ok(true),
            "false" => Ok(false),
            other => Err(NixError::BadOutput(other.to_string())),
        }
    }
}
