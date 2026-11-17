use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::runlog::output;
use super::FacileError;
use crate::catalog::Tool;

/// Runs the binary with `--version` and returns the first line it prints.
///
/// An installer that claims success without running what it placed is lying.
pub fn verify(path: &Path) -> Result<String, FacileError> {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let out = output(Command::new(path).arg("--version"));
    let out = match out {
        Ok(o) if o.status.success() => o,
        _ => {
            return Err(FacileError::Install(format!(
                "{name} was installed but does not run"
            )))
        }
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.trim().lines().next().unwrap_or_default().to_string();
    if line.is_empty() {
        return Err(FacileError::Install(format!("{name} reported no version")));
    }
    Ok(line)
}

/// The version line of `bin` in `bin_dir`, or `None` when it is missing or does not run.
pub fn installed_line(bin_dir: &Path, bin: &str) -> Option<String> {
    let path = bin_dir.join(bin);
    if !path.exists() {
        return None;
    }
    verify(&path).ok()
}

/// Copies `src` to a staged file beside `dest`, then renames it into place.
///
/// Replacing a running binary in place corrupts it; a rename swaps the directory entry instead.
pub(crate) fn atomic_install(src: &Path, dest: &Path) -> Result<(), FacileError> {
    let data = fs::read(src).map_err(FacileError::io("cannot read the built binary"))?;
    let base = dest.file_name().unwrap_or_default().to_string_lossy();
    let dir = dest.parent().unwrap_or(Path::new("."));
    let staged = dir.join(format!(".{base}.new.{}", std::process::id()));
    fs::write(&staged, data).map_err(FacileError::io(format!(
        "cannot write {}",
        staged.display()
    )))?;
    let placed = set_executable(&staged).and_then(|()| fs::rename(&staged, dest));
    if let Err(e) = placed {
        let _ = fs::remove_file(&staged);
        return Err(FacileError::Io {
            context: format!("cannot write {}", dest.display()),
            source: e,
        });
    }
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Creates the bin directory and proves it is writable.
pub(crate) fn prepare_bin_dir(dir: &Path) -> Result<(), FacileError> {
    fs::create_dir_all(dir).map_err(FacileError::io(format!("cannot create {}", dir.display())))?;
    let probe = dir.join(".kit-write-probe");
    fs::write(&probe, b"")
        .map_err(|_| FacileError::Install(format!("{} is not writable", dir.display())))?;
    fs::remove_file(&probe).map_err(FacileError::io(format!("cannot clean {}", probe.display())))
}

/// Places each built binary in tool order, runs each one, and returns the primary's version line.
///
/// A count mismatch means a backend lied about what it built, so nothing is placed.
pub(crate) fn place_all(
    bin_dir: &Path,
    tool: &Tool,
    srcs: &[PathBuf],
) -> Result<String, FacileError> {
    let bins = tool.bin_list();
    if srcs.len() != bins.len() {
        return Err(FacileError::Install(format!(
            "built {} binaries but {} declares {}",
            srcs.len(),
            tool.name,
            bins.len()
        )));
    }
    let mut reported = String::new();
    for (i, (src, bin)) in srcs.iter().zip(&bins).enumerate() {
        let dest = bin_dir.join(bin);
        atomic_install(src, &dest)?;
        let line = verify(&dest)?;
        if i == 0 {
            reported = line;
        }
    }
    Ok(reported)
}

/// Removes every binary of a tool. Nothing to remove counts as removed: returns whether a file went away.
pub fn uninstall(bin_dir: &Path, tool: &Tool) -> Result<bool, FacileError> {
    let mut removed = false;
    for bin in tool.bin_list() {
        let path = bin_dir.join(bin);
        if fs::symlink_metadata(&path).is_err() {
            continue;
        }
        fs::remove_file(&path)
            .map_err(FacileError::io(format!("cannot remove {}", path.display())))?;
        removed = true;
    }
    Ok(removed)
}
