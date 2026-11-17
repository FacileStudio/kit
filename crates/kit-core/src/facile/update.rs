use std::path::{Path, PathBuf};

use super::install::{refuse_foreign, InstallOptions};
use super::owner::Owner;
use super::uptodate::stale;
use super::{Backend, FacileError, Reporter};
use crate::catalog::Tool;

/// How to update a set of tools.
#[derive(Debug, Clone, Default)]
pub struct UpdateOptions {
    /// Reinstall even when the binary already reports the latest tag.
    pub force: bool,
    /// Build from a clone instead of downloading the release.
    pub source: bool,
    /// Skip AI agent skill registration.
    pub no_skill: bool,
    /// Skip `<bin> install` for MCP tools.
    pub no_mcp: bool,
    /// Where binaries live.
    pub bin_dir: PathBuf,
    /// The catalog's `mcp:` list.
    pub mcp_tools: Vec<String>,
}

/// What happened to one tool during an update.
#[derive(Debug)]
pub enum UpdateOutcome {
    /// Installed; the version line the new binary printed.
    Updated(String),
    /// Left alone because another package manager owns it.
    Refused {
        /// Who owns it.
        owner: Owner,
        /// The command that updates it.
        command: String,
    },
    /// The install failed.
    Failed(FacileError),
}

type Results = Vec<(String, UpdateOutcome)>;

fn split_foreign(tools: &[Tool], bin_dir: &Path, r: &dyn Reporter) -> (Vec<Tool>, Results) {
    let mut candidates = Vec::new();
    let mut refused = Vec::new();
    for tool in tools {
        match refuse_foreign(tool, bin_dir) {
            Err(FacileError::Foreign { owner, command, .. }) => {
                r.warn(&format!(
                    "{} is managed by {owner}, kit leaves it alone",
                    tool.name
                ));
                r.hint(&command);
                refused.push((tool.name.clone(), UpdateOutcome::Refused { owner, command }));
            }
            _ => candidates.push(tool.clone()),
        }
    }
    (candidates, refused)
}

impl Backend {
    /// Updates tools. Up-to-date tools are reported and skipped unless `force`; foreign ones are always refused.
    ///
    /// The comparison uses the live latest tag, never the daily cache.
    pub fn update(&self, tools: &[Tool], opts: &UpdateOptions, r: &dyn Reporter) -> Results {
        let (candidates, mut results) = split_foreign(tools, &opts.bin_dir, r);
        let tag_of = |repo: &str| self.github.latest_tag(repo);
        let todo = if opts.force {
            candidates
        } else {
            stale(&candidates, &opts.bin_dir, &tag_of, r)
        };
        let install = InstallOptions {
            source: opts.source,
            no_skill: opts.no_skill,
            no_mcp: opts.no_mcp,
            bin_dir: opts.bin_dir.clone(),
            mcp_tools: opts.mcp_tools.clone(),
            ..InstallOptions::default()
        };
        for tool in todo {
            r.step(&format!("Installing {}", tool.name));
            let outcome = match self.install(&tool, &install, r) {
                Ok(line) => UpdateOutcome::Updated(line),
                Err(e) => UpdateOutcome::Failed(e),
            };
            results.push((tool.name.clone(), outcome));
        }
        results
    }
}
