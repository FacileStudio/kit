//! Finding the flatpak binary.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::{FlathubError, Flatpak};

impl Flatpak {
    /// Finds `flatpak` on the process PATH, or returns `NotAvailable`.
    pub fn from_env() -> Result<Self, FlathubError> {
        Self::locate(&std::env::var_os("PATH").unwrap_or_default())
    }

    /// Finds `flatpak` in a PATH-style list of directories, or returns `NotAvailable`.
    pub fn locate(path_var: &OsStr) -> Result<Self, FlathubError> {
        locate_in(path_var)
            .map(Self::with_binary)
            .ok_or(FlathubError::NotAvailable)
    }

    /// Uses the given binary as `flatpak`.
    pub fn with_binary(bin: impl Into<PathBuf>) -> Self {
        Self { bin: bin.into() }
    }
}

/// Looks for an executable `flatpak` in each directory of a PATH-style value.
fn locate_in(path_var: &OsStr) -> Option<PathBuf> {
    std::env::split_paths(path_var)
        .map(|dir| dir.join("flatpak"))
        .find(|candidate| is_executable(candidate))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}
