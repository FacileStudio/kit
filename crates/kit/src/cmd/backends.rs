use anyhow::Result;
use kit_core::facile::which;
use kit_core::flathub::Flatpak;
use kit_core::listfile::Source;
use kit_core::nix::Nix;

use crate::ui::Ui;

/// The hint for a machine without Nix.
pub const NIX_HINT: &str = "run `kit setup nix` to install Nix";
/// The hint for a machine without Flatpak.
pub const FLATPAK_HINT: &str = "install Flatpak with your distribution's package manager";

/// The flatpak binary on PATH, if any.
pub fn flatpak() -> Option<Flatpak> {
    Flatpak::from_env().ok()
}

/// The Nix handle when `nix` is on PATH.
pub fn nix() -> Result<Option<Nix>> {
    if which("nix", &std::env::var_os("PATH").unwrap_or_default()).is_none() {
        return Ok(None);
    }
    Ok(Some(Nix::new()?))
}

/// The hint that installs a missing backend.
pub fn hint_for(source: Source) -> &'static str {
    match source {
        Source::Nix => NIX_HINT,
        _ => FLATPAK_HINT,
    }
}

/// Warns that `count` listed entries were skipped because their backend is missing.
pub fn skipped(ui: &Ui, source: Source, count: usize) {
    let tool = if source == Source::Nix {
        "nix"
    } else {
        "flatpak"
    };
    let noun = match (source, count) {
        (Source::Nix, 1) => "Nix package",
        (Source::Nix, _) => "Nix packages",
        (_, 1) => "Flathub app",
        _ => "Flathub apps",
    };
    ui.warn(&format!("{tool} is not installed, skipping {count} {noun}"));
    ui.hint(hint_for(source));
}
