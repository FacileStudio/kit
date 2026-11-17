use std::path::Path;
use std::process::Command;

use super::place::installed_line;
use super::runlog::output;
use super::version::version_of;
use super::{FacileError, Reporter};
use crate::catalog::{match_version, Tool};

/// Resolves a repo's latest tag. A seam so update decisions are testable offline.
pub type TagFn<'a> = &'a (dyn Fn(&str) -> Result<String, FacileError> + Sync);

/// True when the installed version line names exactly the latest tag.
///
/// A `dev` build counts as current, so an update never replaces it with a release unasked.
pub fn matches_tag(have: &str, path: &Path, tool: &Tool, tag: &str) -> bool {
    let expected = tag.trim_start_matches('v');
    if have.is_empty() {
        return false;
    }
    let from_pattern = version_from_pattern(have, path, tool);
    if !from_pattern.is_empty() {
        return from_pattern.trim_start_matches('v') == expected;
    }
    if matches_version(have, tool, expected) {
        return true;
    }
    let installed = version_of(have);
    if installed.contains("dev") {
        return true;
    }
    !installed.is_empty() && installed == expected
}

/// True when the installed tool reports the latest tag. An unreadable tag answers no, which reinstalls.
pub fn up_to_date(have: &str, path: &Path, tool: &Tool, tag_of: TagFn) -> bool {
    match tag_of(&tool.repo) {
        Ok(tag) => matches_tag(have, path, tool, &tag),
        Err(_) => false,
    }
}

fn version_from_pattern(have: &str, path: &Path, tool: &Tool) -> String {
    if tool.version_pattern.is_empty() {
        return String::new();
    }
    if tool.version_cmd.is_empty() {
        return match_version(have, &tool.version_pattern);
    }
    match output(Command::new(path).args(tool.version_cmd.split_whitespace())) {
        Ok(out) if out.status.success() => {
            match_version(&String::from_utf8_lossy(&out.stdout), &tool.version_pattern)
        }
        _ => String::new(),
    }
}

fn matches_version(have: &str, tool: &Tool, expected: &str) -> bool {
    let parts: Vec<&str> = have.split_whitespace().collect();
    if parts.len() >= 2 && parts[0] == tool.bin && parts[1] == expected {
        return true;
    }
    parts.len() >= 3 && parts[0] == tool.bin && parts[1] == "version" && parts[2] == expected
}

#[derive(Debug, Default)]
struct Check {
    have: String,
    current: bool,
    err: Option<String>,
}

fn check_one(tool: &Tool, dir: &Path, tag_of: TagFn) -> Check {
    let Some(have) = installed_line(dir, &tool.bin) else {
        return Check::default();
    };
    match tag_of(&tool.repo) {
        Err(e) => Check {
            have,
            current: false,
            err: Some(e.to_string()),
        },
        Ok(tag) => {
            let current = matches_tag(&have, &dir.join(&tool.bin), tool, &tag);
            Check {
                have,
                current,
                err: None,
            }
        }
    }
}

/// The tools that need a download, in input order. Checks run in parallel.
///
/// A missing tool is stale. An installed one whose tag cannot be read is kept: rebuilding a working
/// binary because GitHub was slow helps nobody.
pub fn stale(tools: &[Tool], dir: &Path, tag_of: TagFn, r: &dyn Reporter) -> Vec<Tool> {
    let checks: Vec<Check> = std::thread::scope(|scope| {
        let handles: Vec<_> = tools
            .iter()
            .map(|tool| scope.spawn(move || check_one(tool, dir, tag_of)))
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_default())
            .collect()
    });
    let mut keep = Vec::new();
    for (tool, check) in tools.iter().zip(checks) {
        match (&check.err, check.current) {
            (Some(e), _) => r.warn(&format!(
                "{}: cannot check for an update ({e}), keeping {}",
                tool.name, check.have
            )),
            (None, true) => r.success(&format!("{} is up to date", check.have)),
            (None, false) => keep.push(tool.clone()),
        }
    }
    keep
}
