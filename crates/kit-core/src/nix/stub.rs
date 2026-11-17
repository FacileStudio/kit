use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::Nix;

pub const META: &str = r#"{"locked":{"rev":"abc123"},"revision":"abc123"}"#;

/// A temp dir holding stub `nix` and `nix-env` scripts that log argv and print canned output.
pub struct Stub {
    pub dir: TempDir,
}

impl Stub {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("bin")).unwrap();
        for name in ["nix", "nix-env"] {
            let script = format!(
                "#!/bin/sh\nd={0}\nprintf '%s\\n' {name} \"$@\" >> \"$d/log\"\necho >> \"$d/log\"\n\
                 [ -f \"$d/{name}.err\" ] && cat \"$d/{name}.err\" >&2\n\
                 [ -f \"$d/{name}.out\" ] && cat \"$d/{name}.out\"\n\
                 [ -f \"$d/{name}.code\" ] && exit \"$(cat \"$d/{name}.code\")\"\nexit 0\n",
                dir.path().display()
            );
            let path = dir.path().join("bin").join(name);
            fs::write(&path, script).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        Self { dir }
    }

    pub fn nix(&self) -> Nix {
        let bin = self.dir.path().join("bin");
        Nix::with_binaries(bin.join("nix"), bin.join("nix-env"), self.state())
    }

    pub fn state(&self) -> PathBuf {
        self.dir.path().join("state")
    }

    pub fn set(&self, file: &str, body: &str) {
        fs::write(self.dir.path().join(file), body).unwrap();
    }

    pub fn calls(&self) -> Vec<Vec<String>> {
        let log = fs::read_to_string(self.dir.path().join("log")).unwrap_or_default();
        log.split("\n\n")
            .filter(|c| !c.is_empty())
            .map(|c| c.lines().map(String::from).collect())
            .collect()
    }
}

pub fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| s.to_string()).collect()
}

pub fn path_str(p: &Path) -> String {
    p.display().to_string()
}

pub fn nix_call(rest: &[&str]) -> Vec<String> {
    let head = ["nix", "--extra-experimental-features", "nix-command flakes"];
    head.iter().chain(rest).map(|s| s.to_string()).collect()
}
