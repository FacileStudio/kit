use std::path::{Path, PathBuf};

use super::install::InstallOptions;
use super::owner::{owner_of, Owner};
use super::pathutil::real_path;
use super::version::{is_semver, outdated};
use super::{Backend, FacileError, Reporter};
use crate::catalog::Tool;

/// kit's own repository. kit is not in the catalog: it may live under a package manager's prefix.
pub const KIT_REPO: &str = "FacileStudio/kit";

/// kit described to the installer. The asset matches the release archive `kit_<version>_<os>_<arch>.tar.gz`.
pub fn self_tool() -> Tool {
    Tool {
        name: "kit".into(),
        summary: "The Facile package manager".into(),
        repo: KIT_REPO.into(),
        branch: "main".into(),
        bin: "kit".into(),
        build: "rust".into(),
        src_subdir: ".".into(),
        asset: "kit".into(),
        ..Tool::default()
    }
}

/// How to update kit itself.
#[derive(Debug, Clone, Copy, Default)]
pub struct SelfOptions {
    /// Replace the binary even when it is current or a source build.
    pub force: bool,
    /// Build from a clone instead of downloading the release.
    pub source: bool,
}

/// What a self-update did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelfOutcome {
    /// Replaced; the version line the new binary printed and where it lives.
    Updated(String, PathBuf),
    /// Already at the latest release.
    UpToDate(String),
    /// A source build, which no tag can be ordered against. `force` replaces it.
    SourceBuild(String),
    /// Another package manager owns kit, so kit only says how to update it.
    Managed {
        /// Who owns it.
        owner: Owner,
        /// Whether a newer release exists, as far as kit could tell.
        behind: bool,
        /// The command that updates it.
        command: String,
    },
}

/// The directory holding the running binary, or its owner when that is a package manager.
pub fn self_dir(exe: &Path) -> Result<PathBuf, Owner> {
    let real = real_path(exe);
    let dir = real.parent().map(Path::to_path_buf).unwrap_or_default();
    match owner_of(exe, &dir) {
        owner if owner.is_foreign() => Err(owner),
        _ => Ok(dir),
    }
}

impl Backend {
    /// Replaces the running kit at its own path. Refuses under Homebrew, Nix and mise, naming their command.
    pub fn self_update(
        &self,
        current: &str,
        exe: &Path,
        opts: SelfOptions,
        r: &dyn Reporter,
    ) -> Result<SelfOutcome, FacileError> {
        let dir = match self_dir(exe) {
            Ok(dir) => dir,
            Err(owner) => return Ok(self.managed(owner, current)),
        };
        if !opts.force {
            if !is_semver(current) {
                return Ok(SelfOutcome::SourceBuild(current.to_string()));
            }
            let tag = self.github.latest_tag(KIT_REPO)?;
            if !outdated(current, tag.trim_start_matches('v')) {
                return Ok(SelfOutcome::UpToDate(current.to_string()));
            }
        }
        r.step("Updating kit");
        let install = InstallOptions {
            source: opts.source,
            no_skill: true,
            no_mcp: true,
            bin_dir: dir.clone(),
            ..InstallOptions::default()
        };
        let line = self.install(&self_tool(), &install, r)?;
        Ok(SelfOutcome::Updated(line, dir.join("kit")))
    }

    fn managed(&self, owner: Owner, current: &str) -> SelfOutcome {
        let tag = self.github.latest_tag(KIT_REPO).unwrap_or_default();
        SelfOutcome::Managed {
            owner,
            behind: outdated(current, tag.trim_start_matches('v')),
            command: owner.update_command("kit"),
        }
    }
}
