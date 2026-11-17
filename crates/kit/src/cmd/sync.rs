use anyhow::Result;
use kit_core::facile::{installed_tool, InstallOptions};
use kit_core::listfile::{KitFile, Source};
use kit_core::nix::State as NixState;
use kit_core::paths;
use serde_json::json;

use super::apply::{self, Done, State};
use super::backends;
use super::Ctx;

/// `kit sync`: install what the list names and the machine lacks. Never removes anything.
pub fn run(ctx: &Ctx) -> Result<()> {
    let file = ctx.list()?;
    let mut done = facile(ctx, &file)?;
    done.extend(flathub(ctx, &file.flathub)?);
    done.extend(nix(ctx, &file.nix)?);
    if ctx.ui.json() {
        ctx.ui.emit(&json!({ "file": ctx.file, "changed": done }));
    } else if done.is_empty() {
        ctx.ui.success("Everything listed is installed");
    }
    apply::finish(ctx, &done)
}

fn facile(ctx: &Ctx, file: &KitFile) -> Result<Vec<Done>> {
    if file.facile.is_empty() {
        return Ok(Vec::new());
    }
    let manifest = ctx.catalog()?;
    let backend = ctx.backend()?;
    let opts = apply::facile_opts(ctx, &manifest, InstallOptions::default());
    let mut done = Vec::new();
    for name in &file.facile {
        match manifest.get(name) {
            None => {
                ctx.ui
                    .error(&format!("{name} is not in the Facile catalog"));
                ctx.ui
                    .hint(&format!("run `kit del {name}` to drop it from the list"));
                done.push(Done::new(
                    Source::Facile,
                    name,
                    State::Failed,
                    "not in the catalog",
                ));
            }
            Some(tool) if installed_tool(tool, &ctx.bin_dir).is_some() => {}
            Some(tool) => done.push(apply::install_facile(ctx, &backend, tool, &opts)),
        }
    }
    if !done.is_empty() {
        apply::report_path(ctx);
    }
    Ok(done)
}

fn flathub(ctx: &Ctx, listed: &[String]) -> Result<Vec<Done>> {
    let missing: Vec<String> = match backends::flatpak().filter(|_| !listed.is_empty()) {
        Some(f) => {
            let have = f.list_installed()?;
            listed
                .iter()
                .filter(|id| !have.contains(id))
                .cloned()
                .collect()
        }
        None => listed.to_vec(),
    };
    Ok(apply::install_flathub(ctx, &missing))
}

fn nix(ctx: &Ctx, attrs: &[String]) -> Result<Vec<Done>> {
    let recorded = NixState::load(&paths::state_dir()?)?;
    if attrs.is_empty() && recorded.is_none_or(|s| s.attrs.is_empty()) {
        return Ok(Vec::new());
    }
    let Some(handle) = backends::nix()? else {
        return apply::sync_nix(ctx, attrs, attrs, State::Installed);
    };
    match handle.needs_resync(attrs) {
        Ok(false) => Ok(Vec::new()),
        Ok(true) => apply::sync_nix(ctx, attrs, attrs, State::Installed),
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
