//! Running the flatpak binary and reading its output.

use std::io::ErrorKind;
use std::path::Path;
use std::process::{Command, Stdio};

use super::{validate_id, FlathubError};

const STDERR_TAIL_LINES: usize = 10;

/// Runs `bin` with `args`, stdin closed, and returns stdout on a zero exit.
pub fn run(bin: &Path, args: &[&str]) -> Result<String, FlathubError> {
    let output =
        crate::proc::output(Command::new(bin).args(args).stdin(Stdio::null())).map_err(|err| {
            match err.kind() {
                ErrorKind::NotFound => FlathubError::NotAvailable,
                _ => FlathubError::Io(err),
            }
        })?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    Err(FlathubError::CommandFailed {
        action: args.first().copied().unwrap_or_default().to_string(),
        code: output.status.code(),
        stderr: tail(&String::from_utf8_lossy(&output.stderr)),
    })
}

/// Parses one tab-separated `application\tname` row of `flatpak search`, skipping anything else.
pub fn parse_search_line(line: &str) -> Option<(String, String)> {
    let (id, name) = line.split_once('\t')?;
    let id = id.trim();
    validate_id(id).ok()?;
    Some((id.to_string(), name.trim().to_string()))
}

fn tail(text: &str) -> String {
    let lines: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    let start = lines.len().saturating_sub(STDERR_TAIL_LINES);
    lines[start..].join("\n")
}
