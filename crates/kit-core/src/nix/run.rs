use std::ffi::{OsStr, OsString};
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

use super::NixError;

const TAIL_LINES: usize = 20;
const FEATURES: [&str; 2] = ["--extra-experimental-features", "nix-command flakes"];

/// `args` with the flags that enable `nix-command` and `flakes` in front, so stock installs work.
pub fn flagged<I, S>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    FEATURES
        .into_iter()
        .map(OsString::from)
        .chain(args.into_iter().map(Into::into))
        .collect()
}

/// Runs `bin` with `args` and returns its stdout, mapping a missing binary and a failure to typed errors.
pub fn run<I, S>(bin: &Path, label: &str, args: I) -> Result<String, NixError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let out = crate::proc::output(Command::new(bin).args(args)).map_err(|e| match e.kind() {
        ErrorKind::NotFound => NixError::NotAvailable(bin.display().to_string()),
        _ => NixError::Io(e),
    })?;
    if !out.status.success() {
        return Err(NixError::CommandFailed {
            command: label.to_string(),
            stderr: tail(&out.stderr),
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The last [`TAIL_LINES`] non-empty lines of `stderr`.
pub fn tail(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(TAIL_LINES)..].join("\n")
}
