//! A fake `flatpak` that records its argv, for the backend tests.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use tempfile::TempDir;

use super::Flatpak;

/// A temp dir holding a stub `flatpak` script and what it was called with.
pub struct Stub {
    dir: TempDir,
    pub flatpak: Flatpak,
}

impl Stub {
    /// A stub that prints `stdout` and `stderr`, then exits with `code`.
    pub fn new(stdout: &str, stderr: &str, code: i32) -> Self {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::write(root.join("stdout"), stdout).unwrap();
        fs::write(root.join("stderr"), stderr).unwrap();
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{r}/argv'\ncat '{r}/stdout'\ncat '{r}/stderr' >&2\nexit {code}\n",
            r = root.display()
        );
        let bin = root.join("flatpak");
        fs::write(&bin, script).unwrap();
        fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
        let flatpak = Flatpak::locate(
            &std::env::join_paths([PathBuf::from("/nonexistent"), root.into()]).unwrap(),
        )
        .unwrap();
        Self { dir, flatpak }
    }

    /// A stub that prints nothing and exits 0.
    pub fn ok() -> Self {
        Self::new("", "", 0)
    }

    /// The argv of the last call, or `None` when flatpak never ran.
    pub fn argv(&self) -> Option<Vec<String>> {
        let text = fs::read_to_string(self.dir.path().join("argv")).ok()?;
        Some(text.lines().map(String::from).collect())
    }
}
