use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, Output};
use std::time::Duration;

use super::pathutil::which;
use super::FacileError;

const MAX_FAILURE_LINES: usize = 20;
const TEXT_FILE_BUSY: i32 = 26;

/// Runs a command, retrying briefly while the executable is busy.
///
/// A binary written a moment ago can still be open for writing in a child another thread forked.
pub(crate) fn output(cmd: &mut Command) -> std::io::Result<Output> {
    for _ in 0..50 {
        match cmd.output() {
            Err(e) if e.raw_os_error() == Some(TEXT_FILE_BUSY) => {
                std::thread::sleep(Duration::from_millis(10))
            }
            other => return other,
        }
    }
    cmd.output()
}

/// Runs a build step. On failure the error carries the tail of the log, where the cause and summary sit.
pub(crate) fn run(cmd: &mut Command) -> Result<(), FacileError> {
    let out = output(cmd).map_err(|e| FacileError::Build(format!("build failed: {e}")))?;
    if out.status.success() {
        return Ok(());
    }
    let mut log = String::from_utf8_lossy(&out.stdout).to_string();
    log.push_str(&String::from_utf8_lossy(&out.stderr));
    let detail = last_lines(&log, MAX_FAILURE_LINES);
    if detail.is_empty() {
        return Err(FacileError::Build(format!("build failed: {}", out.status)));
    }
    Err(FacileError::Build(format!("build failed:\n{detail}")))
}

/// At most `n` trailing lines of a build log, trimmed.
pub(crate) fn last_lines(out: &str, n: usize) -> String {
    let lines: Vec<&str> = out.trim_end_matches('\n').split('\n').collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n").trim().to_string()
}

/// An error naming the missing command and how to get it.
pub(crate) fn need(bin: &str, advice: &str, path_var: &OsStr) -> Result<(), FacileError> {
    match which(bin, path_var) {
        Some(_) => Ok(()),
        None => Err(FacileError::Build(format!("{bin} not found: {advice}"))),
    }
}

/// `MISE_IGNORED_CONFIG_PATHS` with the checkout appended, so mise shims skip the clone's own mise.toml.
///
/// An untrusted config makes every shim abort, and trusting a fresh clone would run its hooks.
pub(crate) fn ignore_repo_mise(existing: Option<OsString>, checkout: &Path) -> OsString {
    match existing {
        Some(paths) if !paths.is_empty() => {
            let mut all: Vec<_> = std::env::split_paths(&paths).collect();
            all.push(checkout.to_path_buf());
            std::env::join_paths(all).unwrap_or(paths)
        }
        _ => checkout.as_os_str().to_os_string(),
    }
}

/// A command run in `dir` with the repo's mise config ignored.
pub(crate) fn build_command(program: &str, args: &[&str], dir: &Path, checkout: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.args(args).current_dir(dir).env(
        "MISE_IGNORED_CONFIG_PATHS",
        ignore_repo_mise(std::env::var_os("MISE_IGNORED_CONFIG_PATHS"), checkout),
    );
    cmd
}
