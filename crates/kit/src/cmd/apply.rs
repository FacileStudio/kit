use anyhow::Result;
use kit_core::catalog::{Manifest, Tool};
use kit_core::facile::{on_path, Backend, InstallOptions};
use kit_core::listfile::Source;
use serde::Serialize;

use super::backends;
use super::Ctx;
use crate::ui::Failure;

/// What happened to one package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// Installed or rebuilt by this run.
    Installed,
    /// Removed by this run.
    Removed,
    /// Its backend is not on this machine.
    Skipped,
    /// The backend failed.
    Failed,
}

/// One package and what this run did to it.
#[derive(Debug, Clone, Serialize)]
pub struct Done {
    /// The backend name.
    pub source: &'static str,
    /// The list file ID.
    pub id: String,
    /// The outcome.
    pub state: State,
    /// The version line or error text.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub detail: String,
}

impl Done {
    /// A record for `id` in `source`.
    pub fn new(source: Source, id: &str, state: State, detail: impl Into<String>) -> Done {
        Done {
            source: source.as_str(),
            id: id.to_string(),
            state,
            detail: detail.into(),
        }
    }
}

/// The install options for Facile tools, with the catalog's MCP list.
pub fn facile_opts(ctx: &Ctx, manifest: &Manifest, base: InstallOptions) -> InstallOptions {
    InstallOptions {
        bin_dir: ctx.bin_dir.clone(),
        mcp_tools: manifest.mcp.clone(),
        ..base
    }
}

/// Installs one Facile tool and reports it.
pub fn install_facile(ctx: &Ctx, backend: &Backend, tool: &Tool, opts: &InstallOptions) -> Done {
    ctx.ui.step(&format!("Installing {}", tool.name));
    match backend.install(tool, opts, ctx.ui) {
        Ok(line) => {
            let place = format!("{}/{}", ctx.tilde(&ctx.bin_dir), tool.bin_list().join(" "));
            ctx.ui.success(&format!("{line} installed to {place}"));
            Done::new(Source::Facile, &tool.name, State::Installed, line)
        }
        Err(e) => {
            ctx.ui.error(&format!("{}: {e}", tool.name));
            Done::new(Source::Facile, &tool.name, State::Failed, e.to_string())
        }
    }
}

/// Installs Flathub apps at user scope after making sure the remote exists.
pub fn install_flathub(ctx: &Ctx, ids: &[String]) -> Vec<Done> {
    let mark = |state, detail: &str| {
        ids.iter()
            .map(|id| Done::new(Source::Flathub, id, state, detail))
            .collect()
    };
    if ids.is_empty() {
        return Vec::new();
    }
    let Some(flatpak) = backends::flatpak() else {
        backends::skipped(ctx.ui, Source::Flathub, ids.len());
        return mark(State::Skipped, "flatpak is not installed");
    };
    ctx.ui
        .step(&format!("Installing {} from Flathub", ids.join(", ")));
    let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    match flatpak
        .ensure_remote()
        .and_then(|()| flatpak.install(&refs))
    {
        Ok(()) => {
            ctx.ui
                .success(&format!("{} installed from Flathub", ids.join(", ")));
            mark(State::Installed, "")
        }
        Err(e) => {
            ctx.ui.error(&e.to_string());
            mark(State::Failed, &e.to_string())
        }
    }
}

/// Rebuilds the kit Nix profile from `attrs`. `changed` are the entries this run is about.
pub fn sync_nix(
    ctx: &Ctx,
    attrs: &[String],
    changed: &[String],
    state: State,
) -> Result<Vec<Done>> {
    let mark = |state, detail: &str| {
        changed
            .iter()
            .map(|id| Done::new(Source::Nix, id, state, detail))
            .collect()
    };
    let Some(nix) = backends::nix()? else {
        backends::skipped(ctx.ui, Source::Nix, changed.len().max(1));
        return Ok(mark(State::Skipped, "nix is not installed"));
    };
    ctx.ui.step(&format!(
        "Building the Nix profile with {} package(s)",
        attrs.len()
    ));
    match nix.sync(attrs) {
        Ok(built) => {
            ctx.ui.success(&format!(
                "Nix profile built from nixpkgs {}",
                short(&built.rev)
            ));
            Ok(mark(state, ""))
        }
        Err(e) => {
            ctx.ui.error(&e.to_string());
            Ok(mark(State::Failed, &e.to_string()))
        }
    }
}

fn short(rev: &str) -> &str {
    rev.get(..12).unwrap_or(rev)
}

/// Warns, never fixes, when the bin directory is not on PATH.
pub fn report_path(ctx: &Ctx) {
    if on_path(&ctx.bin_dir, &std::env::var_os("PATH").unwrap_or_default()) {
        return;
    }
    ctx.ui
        .warn(&format!("{} is not on your PATH", ctx.tilde(&ctx.bin_dir)));
    ctx.ui
        .hint(&format!("export PATH=\"{}:$PATH\"", ctx.bin_dir.display()));
}

/// Fails with one summary line when any package failed. Skipped ones are not failures.
pub fn finish(ctx: &Ctx, done: &[Done]) -> Result<()> {
    let failed: Vec<&str> = done
        .iter()
        .filter(|d| d.state == State::Failed)
        .map(|d| d.id.as_str())
        .collect();
    if failed.is_empty() {
        return Ok(());
    }
    if !ctx.ui.json() {
        ctx.ui.error(&format!(
            "{} of {} packages failed: {}",
            failed.len(),
            done.len(),
            failed.join(", ")
        ));
    }
    Err(Failure::silent(1).into())
}
