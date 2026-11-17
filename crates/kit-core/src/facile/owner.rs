use std::fmt;
use std::path::Path;

use serde::Serialize;

use super::pathutil::real_path;

/// Who put a binary where it is. Only kit-owned binaries may be replaced by kit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Owner {
    /// A file kit placed in its bin directory.
    Kit,
    /// A Nix store path, read-only and rebuilt by Nix.
    Nix,
    /// A mise install directory or shim.
    Mise,
    /// A Homebrew cask or formula, which brew restores on its next upgrade.
    Homebrew,
    /// Anywhere else, such as a symlink to a local build.
    Other,
}

impl fmt::Display for Owner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Owner::Kit => "kit",
            Owner::Nix => "Nix",
            Owner::Mise => "mise",
            Owner::Homebrew => "Homebrew",
            Owner::Other => "something other than kit",
        };
        f.write_str(name)
    }
}

impl Owner {
    /// True when kit must not overwrite the binary: another package manager would undo it or fight it.
    pub fn is_foreign(self) -> bool {
        matches!(self, Owner::Nix | Owner::Mise | Owner::Homebrew)
    }

    /// The command that updates a binary called `name` under this owner. `name` is `kit` for kit itself.
    pub fn update_command(self, name: &str) -> String {
        match (self, name) {
            (Owner::Homebrew, "kit") => "brew upgrade --cask kit".to_string(),
            (Owner::Homebrew, _) => format!("brew upgrade {name}"),
            (Owner::Nix, _) => "nixos-rebuild switch, or home-manager switch".to_string(),
            (Owner::Mise, _) => "mise upgrade".to_string(),
            (_, "kit") => "kit self-update".to_string(),
            _ => format!("kit update {name}"),
        }
    }
}

/// Who owns the binary at `path`, given kit's bin directory. Symlinks are resolved first.
pub fn owner_of(path: &Path, bin_dir: &Path) -> Owner {
    let resolved = real_path(path);
    let raw = path.to_string_lossy();
    let real = resolved.to_string_lossy();
    if from_homebrew(&real) {
        return Owner::Homebrew;
    }
    if real.starts_with("/nix/store/") {
        return Owner::Nix;
    }
    if from_mise(&raw) || from_mise(&real) {
        return Owner::Mise;
    }
    match resolved.parent() {
        Some(dir) if real_path(dir) == real_path(bin_dir) => Owner::Kit,
        _ => Owner::Other,
    }
}

/// True for a path a Homebrew cask or formula resolves to. A checked-out tap is not an install.
pub fn from_homebrew(path: &str) -> bool {
    ["/Caskroom/", "/Cellar/"]
        .iter()
        .any(|dir| path.contains(dir))
        || ["/opt/homebrew/", "/usr/local/Homebrew/", "/home/linuxbrew/"]
            .iter()
            .any(|prefix| path.starts_with(prefix))
}

fn from_mise(path: &str) -> bool {
    path.contains("/mise/installs/") || path.contains("/mise/shims/")
}
