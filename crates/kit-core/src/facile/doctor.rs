use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;

use serde::Serialize;

use super::pathutil::{on_path, real_path, tilde, which};
use super::status::installed_tool;
use super::Backend;
use crate::catalog::Manifest;

/// One problem doctor found, with the commands or paths that fix it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    /// What is wrong.
    pub message: String,
    /// What to run or look at.
    pub hints: Vec<String>,
}

fn finding(message: String, hints: Vec<String>) -> Option<Finding> {
    Some(Finding { message, hints })
}

/// The bin directory is missing or not on PATH. kit prints the PATH line and never edits shell config.
pub fn check_bin_dir(dir: &Path, path_var: &OsStr, home: &Path) -> Option<Finding> {
    if !dir.exists() {
        return finding(
            format!("{} does not exist yet", tilde(dir, home)),
            Vec::new(),
        );
    }
    if !on_path(dir, path_var) {
        let hint = format!("export PATH=\"{}:$PATH\"", dir.display());
        return finding(
            format!("{} is not on your PATH", tilde(dir, home)),
            vec![hint],
        );
    }
    None
}

/// Another binary of the same name comes first on PATH.
pub fn check_shadow(bin: &str, dir: &Path, path_var: &OsStr) -> Option<Finding> {
    let found = which(bin, path_var)?;
    if real_path(&found) == real_path(&dir.join(bin)) {
        return None;
    }
    finding(
        format!(
            "another {bin} comes first on your PATH: {}",
            found.display()
        ),
        Vec::new(),
    )
}

/// Staged `.<bin>.new.<pid>` files an interrupted install left behind.
pub fn check_staged(dir: &Path, home: &Path) -> Option<Finding> {
    let mut hints: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| is_staged(p))
        .map(|p| format!("rm {}", p.display()))
        .collect();
    if hints.is_empty() {
        return None;
    }
    hints.sort();
    let message = format!(
        "{} interrupted install(s) left staged files in {}",
        hints.len(),
        tilde(dir, home)
    );
    finding(message, hints)
}

fn is_staged(path: &Path) -> bool {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    name.starts_with('.') && name.contains(".new.")
}

/// Other copies of `name` on PATH besides the running one.
pub fn check_copies(name: &str, exe: &Path, path_var: &OsStr) -> Option<Finding> {
    let mut seen: HashSet<_> = HashSet::from([real_path(exe)]);
    let others: Vec<String> = std::env::split_paths(path_var)
        .map(|dir| dir.join(name))
        .filter(|p| p.is_file() && seen.insert(real_path(p)))
        .map(|p| p.display().to_string())
        .collect();
    if others.is_empty() {
        return None;
    }
    finding(
        format!("{} other {name} binaries on your PATH", others.len()),
        others,
    )
}

impl Backend {
    /// Every local problem: the bin directory, shadowed tools, interrupted installs and stray kit copies.
    pub fn doctor(&self, manifest: &Manifest, bin_dir: &Path, exe: &Path) -> Vec<Finding> {
        let path = &self.env.path;
        let home = &self.env.home;
        let mut found: Vec<Finding> = check_bin_dir(bin_dir, path, home).into_iter().collect();
        let installed: Vec<_> = manifest
            .tools
            .iter()
            .filter(|t| installed_tool(t, bin_dir).is_some())
            .collect();
        if installed.is_empty() {
            found.extend(finding(
                "no Facile tools installed".into(),
                vec!["kit add <tool>".into()],
            ));
        }
        found.extend(
            installed
                .iter()
                .filter_map(|t| check_shadow(&t.bin, bin_dir, path)),
        );
        found.extend(check_staged(bin_dir, home));
        found.extend(check_copies("kit", exe, path));
        found
    }
}
