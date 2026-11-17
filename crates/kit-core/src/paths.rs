//! Where kit keeps things on disk. Every other module asks here.

use std::env;
use std::path::PathBuf;

/// The home directory, or an error naming what to set.
pub fn home() -> std::io::Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| std::io::Error::other("no home directory: set HOME"))
}

/// The directory Facile tool binaries go to: `KIT_BIN_DIR`, then `FACILE_BIN_DIR`, then `~/.local/bin`.
pub fn bin_dir() -> std::io::Result<PathBuf> {
    if let Some(dir) = env::var_os("KIT_BIN_DIR").or_else(|| env::var_os("FACILE_BIN_DIR")) {
        return Ok(PathBuf::from(dir));
    }
    Ok(home()?.join(".local").join("bin"))
}

/// `$XDG_CONFIG_HOME/kit`, or `~/.config/kit`.
pub fn config_dir() -> std::io::Result<PathBuf> {
    Ok(xdg("XDG_CONFIG_HOME", ".config")?.join("kit"))
}

/// `$XDG_CACHE_HOME/kit`, or `~/.cache/kit`.
pub fn cache_dir() -> std::io::Result<PathBuf> {
    Ok(xdg("XDG_CACHE_HOME", ".cache")?.join("kit"))
}

/// `$XDG_STATE_HOME/kit`, or `~/.local/state/kit`.
pub fn state_dir() -> std::io::Result<PathBuf> {
    Ok(xdg("XDG_STATE_HOME", ".local/state")?.join("kit"))
}

/// The system-wide seed list a distribution may ship.
pub fn system_seed() -> PathBuf {
    PathBuf::from("/etc/kit/kit.yml")
}

fn xdg(var: &str, fallback: &str) -> std::io::Result<PathBuf> {
    match env::var_os(var) {
        Some(dir) if !dir.is_empty() => Ok(PathBuf::from(dir)),
        _ => Ok(home()?.join(fallback)),
    }
}
