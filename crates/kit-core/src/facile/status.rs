use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Serialize;

use super::owner::{owner_of, Owner};
use super::place::installed_line;
use super::selfupdate::self_tool;
use super::version::{outdated, version_of};
use super::Backend;
use crate::catalog::{Manifest, Tool};

/// A Facile tool found by running its binary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InstalledTool {
    /// The catalog name.
    pub name: String,
    /// Where the primary binary is.
    pub path: PathBuf,
    /// The first line `--version` printed.
    pub line: String,
    /// The version part of that line.
    pub version: String,
    /// Who placed it.
    pub owner: Owner,
}

/// One listing row: a catalog tool, what is installed, and the latest tag from the daily cache.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolStatus {
    /// The catalog name.
    pub name: String,
    /// One line on what the tool does.
    pub summary: String,
    /// The GitHub `owner/repo`.
    pub repo: String,
    /// The installed binary, if it runs.
    pub installed: Option<InstalledTool>,
    /// The latest tag without its `v`, empty when unknown or not asked.
    pub latest: String,
    /// True only when the installed version orders strictly before the latest.
    pub outdated: bool,
}

/// The tool if its primary binary in `bin_dir` runs.
pub fn installed_tool(tool: &Tool, bin_dir: &Path) -> Option<InstalledTool> {
    let line = installed_line(bin_dir, &tool.bin)?;
    let path = bin_dir.join(&tool.bin);
    Some(InstalledTool {
        name: tool.name.clone(),
        owner: owner_of(&path, bin_dir),
        version: version_of(&line),
        path,
        line,
    })
}

impl Backend {
    /// Every catalog tool whose binary runs, in catalog order. There is no state file: running is proof.
    pub fn installed(&self, manifest: &Manifest, bin_dir: &Path) -> Vec<InstalledTool> {
        std::thread::scope(|scope| {
            let handles: Vec<_> = manifest
                .tools
                .iter()
                .map(|tool| scope.spawn(move || installed_tool(tool, bin_dir)))
                .collect();
            handles
                .into_iter()
                .filter_map(|h| h.join().ok().flatten())
                .collect()
        })
    }

    /// kit first, then every catalog tool. Latest tags are asked for installed tools only, through the cache.
    pub fn statuses(
        &self,
        manifest: &Manifest,
        bin_dir: &Path,
        kit_version: &str,
        now: SystemTime,
    ) -> Vec<ToolStatus> {
        let installed = self.installed(manifest, bin_dir);
        let mut repos = vec![self_tool().repo];
        repos.extend(
            manifest
                .tools
                .iter()
                .filter(|t| installed.iter().any(|i| i.name == t.name))
                .map(|t| t.repo.clone()),
        );
        let latest = self.latest_cache().resolve(&self.github, &repos, now);
        let kit = self_tool();
        let mut rows = vec![row(&kit, Some(self_row(kit_version)), &latest)];
        for tool in &manifest.tools {
            let found = installed.iter().find(|i| i.name == tool.name).cloned();
            rows.push(row(tool, found, &latest));
        }
        rows
    }
}

fn self_row(version: &str) -> InstalledTool {
    let path = std::env::current_exe().unwrap_or_default();
    InstalledTool {
        name: "kit".into(),
        line: format!("kit {version}"),
        version: version.to_string(),
        owner: super::selfupdate::self_dir(&path)
            .err()
            .unwrap_or(Owner::Kit),
        path,
    }
}

fn row(
    tool: &Tool,
    installed: Option<InstalledTool>,
    latest: &std::collections::HashMap<String, String>,
) -> ToolStatus {
    let latest = match &installed {
        Some(_) => latest
            .get(&tool.repo)
            .map(|t| t.trim_start_matches('v').to_string())
            .unwrap_or_default(),
        None => String::new(),
    };
    let is_outdated = installed
        .as_ref()
        .is_some_and(|i| outdated(&i.version, &latest));
    ToolStatus {
        name: tool.name.clone(),
        summary: tool.summary.clone(),
        repo: tool.repo.clone(),
        installed,
        latest,
        outdated: is_outdated,
    }
}
