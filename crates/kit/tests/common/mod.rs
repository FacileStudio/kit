//! A sandboxed home, a fake catalog and stub `flatpak` and `nix` binaries for the CLI tests.

#![allow(dead_code)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use tempfile::TempDir;

mod out;

pub use out::{json, stderr, stdout};

const CATALOG: &str = "version: 1\ntools:\n  - name: sablier\n    summary: Time tracking\n    repo: FacileStudio/sablier-cli\n    bin: sablier\n    asset: sablier\n  - name: nuage\n    summary: Cloud storage\n    repo: FacileStudio/nuage-cli\n    bin: nuage\n    asset: nuage\n";

const FLATPAK: &str = r#"#!/bin/sh
echo "flatpak $*" >> "$KIT_TEST_LOG"
case "$1" in
  search) printf 'org.gnome.Calculator\tCalculator\nio.github.calc.Calc\tCalculator\norg.gimp.GIMP\tGIMP\n' ;;
  list) cat "$KIT_TEST_DIR/installed" 2>/dev/null ;;
  install) shift; for a in "$@"; do case "$a" in -*|flathub) ;; *) echo "$a" >> "$KIT_TEST_DIR/installed" ;; esac; done ;;
esac
exit 0
"#;

const NIX: &str = r#"#!/bin/sh
echo "nix $*" >> "$KIT_TEST_LOG"
case "$*" in
  *"flake metadata"*) echo '{"locked":{"rev":"abc123"}}' ;;
  *eval*) case "$*" in *ripgrep*) echo true ;; *) echo false ;; esac ;;
esac
exit 0
"#;

/// One test's machine.
pub struct Sandbox {
    pub dir: TempDir,
}

impl Sandbox {
    /// A home with the fake catalog, a fresh latest-tag cache and no flatpak or nix.
    pub fn new() -> Sandbox {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::create_dir_all(root.join("home/.cache/kit")).unwrap();
        fs::write(root.join("catalog.yml"), CATALOG).unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let latest = format!(
            r#"{{"FacileStudio/kit":{{"tag":"v{}","checked":{now}}}}}"#,
            env!("CARGO_PKG_VERSION")
        );
        fs::write(root.join("home/.cache/kit/latest.json"), latest).unwrap();
        Sandbox { dir }
    }

    /// Puts stub `flatpak`, `nix` and `nix-env` on PATH.
    pub fn with_backends(self) -> Sandbox {
        self.stub("flatpak", FLATPAK);
        self.stub("nix", NIX);
        self.stub(
            "nix-env",
            "#!/bin/sh\necho \"nix-env $*\" >> \"$KIT_TEST_LOG\"\n",
        );
        self
    }

    fn stub(&self, name: &str, script: &str) {
        let path = self.dir.path().join("bin").join(name);
        fs::write(&path, script).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    /// Runs kit with `args` in the sandbox, stdin closed.
    pub fn run(&self, args: &[&str]) -> Output {
        let root = self.dir.path();
        let path = format!("{}:/usr/bin:/bin", root.join("bin").display());
        Command::new(env!("CARGO_BIN_EXE_kit"))
            .args(args)
            .env_clear()
            .env("HOME", root.join("home"))
            .env("PATH", path)
            .env("KIT_CATALOG", root.join("catalog.yml"))
            .env("KIT_TEST_LOG", root.join("log"))
            .env("KIT_TEST_DIR", root)
            .env("NO_COLOR", "1")
            .stdin(Stdio::null())
            .output()
            .unwrap()
    }

    /// The list file path.
    pub fn file(&self) -> PathBuf {
        self.dir.path().join("home/.config/kit/kit.yml")
    }

    /// Writes the list file.
    pub fn write_file(&self, text: &str) {
        fs::create_dir_all(self.file().parent().unwrap()).unwrap();
        fs::write(self.file(), text).unwrap();
    }

    /// What the list file says now.
    pub fn read_file(&self) -> String {
        fs::read_to_string(self.file()).unwrap_or_default()
    }

    /// Every stub call so far, one per line.
    pub fn log(&self) -> String {
        fs::read_to_string(self.dir.path().join("log")).unwrap_or_default()
    }
}
