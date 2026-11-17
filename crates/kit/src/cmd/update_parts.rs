use anyhow::Result;
use kit_core::catalog::{Manifest, Tool};
use kit_core::facile::{Backend, UpdateOptions, UpdateOutcome};
use kit_core::listfile::{KitFile, Source};

use super::apply::{Done, State};
use super::backends;
use super::del::find;
use super::Ctx;

/// What an update touches.
#[derive(Debug, Default)]
pub struct Targets {
    /// Facile tools, in list order.
    pub facile: Vec<Tool>,
    /// Whether to update the Flathub apps.
    pub flathub: bool,
    /// Whether to refresh the Nix profile.
    pub nix: bool,
}

/// Everything listed when no name was given, else exactly the named packages.
pub fn targets(
    file: &KitFile,
    manifest: &Manifest,
    names: &[String],
    named: bool,
) -> Result<Targets> {
    if !named {
        return Ok(Targets {
            facile: file
                .facile
                .iter()
                .filter_map(|n| manifest.get(n).cloned())
                .collect(),
            flathub: !file.flathub.is_empty(),
            nix: !file.nix.is_empty(),
        });
    }
    let mut out = Targets::default();
    for name in names {
        match find(file, manifest, name)? {
            (Source::Facile, id) => out.facile.extend(manifest.get(&id).cloned()),
            (Source::Flathub, _) => out.flathub = true,
            (Source::Nix, _) => out.nix = true,
        }
    }
    Ok(out)
}

/// Runs the Facile update and turns each outcome into a record.
pub fn facile(ctx: &Ctx, backend: &Backend, tools: &[Tool], opts: &UpdateOptions) -> Vec<Done> {
    let mut done = Vec::new();
    for (name, outcome) in backend.update(tools, opts, ctx.ui) {
        done.push(match outcome {
            UpdateOutcome::Updated(line) => {
                ctx.ui
                    .success(&format!("{line} installed to {}", ctx.tilde(&ctx.bin_dir)));
                Done::new(Source::Facile, &name, State::Installed, line)
            }
            UpdateOutcome::Refused { command, .. } => {
                Done::new(Source::Facile, &name, State::Skipped, command)
            }
            UpdateOutcome::Failed(e) => {
                ctx.ui.error(&format!("{name}: {e}"));
                Done::new(Source::Facile, &name, State::Failed, e.to_string())
            }
        });
    }
    done
}

/// Updates every app and runtime in the user Flatpak installation.
pub fn flathub(ctx: &Ctx, listed: usize) -> Vec<Done> {
    let Some(flatpak) = backends::flatpak() else {
        backends::skipped(ctx.ui, Source::Flathub, listed);
        return vec![Done::new(
            Source::Flathub,
            "apps",
            State::Skipped,
            "flatpak is not installed",
        )];
    };
    ctx.ui.step("Updating Flathub apps");
    match flatpak.update() {
        Ok(()) => {
            ctx.ui.success("Flathub apps are up to date");
            vec![Done::new(Source::Flathub, "apps", State::Installed, "")]
        }
        Err(e) => {
            ctx.ui.error(&e.to_string());
            vec![Done::new(
                Source::Flathub,
                "apps",
                State::Failed,
                e.to_string(),
            )]
        }
    }
}

/// Rebuilds the Nix profile when the registry nixpkgs moved, or always with `force`.
pub fn nix(ctx: &Ctx, attrs: &[String], force: bool) -> Result<Vec<Done>> {
    let Some(handle) = backends::nix()? else {
        backends::skipped(ctx.ui, Source::Nix, attrs.len());
        return Ok(vec![Done::new(
            Source::Nix,
            "profile",
            State::Skipped,
            "nix is not installed",
        )]);
    };
    match handle.needs_resync(attrs) {
        Ok(false) if !force => {
            ctx.ui.success("Nix profile is up to date");
            Ok(Vec::new())
        }
        Ok(_) => super::apply::sync_nix(ctx, attrs, &["profile".to_string()], State::Installed),
        Err(e) => {
            ctx.ui.error(&e.to_string());
            Ok(vec![Done::new(
                Source::Nix,
                "profile",
                State::Failed,
                e.to_string(),
            )])
        }
    }
}
