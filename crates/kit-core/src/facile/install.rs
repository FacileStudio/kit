use std::path::{Path, PathBuf};

use super::owner::owner_of;
use super::pathutil::which;
use super::place::{place_all, prepare_bin_dir};
use super::{Backend, FacileError, Reporter};
use crate::catalog::Tool;

/// How to install one tool.
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Build from a clone instead of downloading the release.
    pub source: bool,
    /// The tag to install. Empty means the latest release.
    pub version: String,
    /// Skip AI agent skill registration.
    pub no_skill: bool,
    /// Skip `<bin> install` for MCP tools.
    pub no_mcp: bool,
    /// Where binaries go.
    pub bin_dir: PathBuf,
    /// Replace a binary in the bin directory even when Nix, mise or Homebrew put it there.
    pub force: bool,
    /// Tools whose `install` subcommand registers an MCP server: the catalog's `mcp:` list.
    pub mcp_tools: Vec<String>,
}

type ReleaseFn<'a> = &'a dyn Fn(&Tool, &str, &Path) -> Result<Vec<PathBuf>, FacileError>;

/// One build: the tool, its options and the scratch directory.
pub(crate) struct Job<'a> {
    pub(crate) tool: &'a Tool,
    pub(crate) opts: &'a InstallOptions,
    pub(crate) work: &'a Path,
}

impl Backend {
    /// Installs every binary of a tool and returns the version line the installed primary prints.
    pub fn install(
        &self,
        tool: &Tool,
        opts: &InstallOptions,
        r: &dyn Reporter,
    ) -> Result<String, FacileError> {
        prepare_bin_dir(&opts.bin_dir)?;
        if !opts.force {
            refuse_foreign(tool, &opts.bin_dir)?;
        }
        let work = tempfile::Builder::new()
            .prefix("kit-")
            .tempdir()
            .map_err(FacileError::io("cannot create a work directory"))?;
        check_requires(tool, &self.env.path)?;
        let release = |t: &Tool, v: &str, w: &Path| self.download_release(t, v, w, r);
        let job = Job {
            tool,
            opts,
            work: work.path(),
        };
        let srcs = self.build(&job, &release, r)?;
        let reported = place_all(&opts.bin_dir, tool, &srcs)?;
        if !opts.no_skill {
            self.register_skill(tool, work.path(), r);
        }
        if !opts.no_mcp && opts.mcp_tools.iter().any(|t| t == &tool.name) {
            register_mcp(&opts.bin_dir.join(&tool.bin_list()[0]), r);
        }
        Ok(reported)
    }

    /// Prefers the release and falls back to a source build, except after an integrity failure.
    pub(crate) fn build(
        &self,
        job: &Job,
        release: ReleaseFn,
        r: &dyn Reporter,
    ) -> Result<Vec<PathBuf>, FacileError> {
        let Job { tool, opts, work } = *job;
        if !tool.asset.is_empty() && !opts.source {
            match release(tool, &opts.version, work) {
                Ok(paths) => return Ok(paths),
                Err(e) if e.is_integrity() => return Err(e),
                Err(e) => r.warn(&format!("falling back to a source build: {e}")),
            }
        }
        self.build_source(tool, &opts.version, work, r)
    }
}

/// Fails when a binary of the tool already in `bin_dir` belongs to another package manager.
pub(crate) fn refuse_foreign(tool: &Tool, bin_dir: &Path) -> Result<(), FacileError> {
    for bin in tool.bin_list() {
        let path = bin_dir.join(&bin);
        if std::fs::symlink_metadata(&path).is_err() {
            continue;
        }
        let owner = owner_of(&path, bin_dir);
        if owner.is_foreign() {
            return Err(FacileError::Foreign {
                command: owner.update_command(&tool.name),
                name: bin,
                owner,
            });
        }
    }
    Ok(())
}

/// Runs `<bin> install` so the tool registers its MCP server. A failure warns and keeps the binary.
pub(crate) fn register_mcp(dest: &Path, r: &dyn Reporter) {
    let out = super::runlog::output(std::process::Command::new(dest).arg("install"));
    let detail = match out {
        Ok(o) if o.status.success() => return,
        Ok(o) => format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        ),
        Err(e) => e.to_string(),
    };
    let name = dest.file_name().unwrap_or_default().to_string_lossy();
    r.warn(&format!(
        "could not register {name}'s MCP server: {}",
        detail.trim()
    ));
    r.hint(&format!("run {} install to retry", dest.display()));
}

fn check_requires(tool: &Tool, path: &std::ffi::OsStr) -> Result<(), FacileError> {
    match tool.requires.iter().find(|req| which(req, path).is_none()) {
        Some(req) => Err(FacileError::Install(format!(
            "{} needs {req} on your PATH: install it first",
            tool.name
        ))),
        None => Ok(()),
    }
}
