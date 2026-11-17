use std::path::{Path, PathBuf};
use std::process::Command;

use super::build::{build_for, Checkout};
use super::runlog::need;
use super::{Backend, FacileError, Reporter};
use crate::catalog::Tool;

fn toolchain(build: &str) -> Option<(&'static str, &'static str)> {
    match build {
        "rust" => Some(("cargo", "install Rust from https://rustup.rs")),
        "go" => Some(("go", "install Go from https://go.dev/dl")),
        "bun" => Some(("bun", "install Bun from https://bun.sh")),
        _ => None,
    }
}

impl Backend {
    /// Builds the tool from a clone. A named version clones that tag, so a missing archive still installs it.
    pub(crate) fn build_source(
        &self,
        tool: &Tool,
        version: &str,
        work: &Path,
        r: &dyn Reporter,
    ) -> Result<Vec<PathBuf>, FacileError> {
        let Some((bin, advice)) = toolchain(&tool.build) else {
            return Err(FacileError::Build(format!(
                "unknown build backend: {}",
                tool.build
            )));
        };
        let version = match version {
            "" => self.github.latest_tag(&tool.repo).unwrap_or_default(),
            v => v.to_string(),
        };
        need("git", "install git first", &self.env.path)?;
        need(bin, advice, &self.env.path)?;
        let root = checkout_source(tool, &version, work, r)?;
        r.step("Building from source, this takes a minute");
        let out = work.join("out");
        std::fs::create_dir_all(&out)
            .map_err(FacileError::io(format!("cannot create {}", out.display())))?;
        let src = root.join(&tool.src_subdir);
        build_for(
            tool,
            &Checkout {
                src: &src,
                root: &root,
                out: &out,
            },
        )
    }
}

fn checkout_source(
    tool: &Tool,
    version: &str,
    work: &Path,
    r: &dyn Reporter,
) -> Result<PathBuf, FacileError> {
    let reference = if version.is_empty() {
        tool.branch.as_str()
    } else {
        version
    };
    r.step("Fetching source");
    let checkout = work.join("src");
    let url = format!("https://github.com/{}.git", tool.repo);
    let out = Command::new("git")
        .args([
            "clone", "--depth", "1", "--quiet", "--branch", reference, &url,
        ])
        .arg(&checkout)
        .output()
        .map_err(|e| FacileError::Build(format!("cannot run git: {e}")))?;
    if !out.status.success() {
        let detail = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(FacileError::Build(format!(
            "cannot clone {} at {reference}: {detail}",
            tool.repo
        )));
    }
    Ok(checkout)
}
