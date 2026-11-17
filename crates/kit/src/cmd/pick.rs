use std::cell::OnceCell;

use anyhow::{bail, Result};
use kit_core::catalog::{Manifest, Tool};
use kit_core::facile::installed_tool;
use kit_core::flathub::FlathubError;
use kit_core::listfile::Source;
use kit_core::nix::NixError;
use kit_core::resolve::{resolve, Candidate, Context, Resolution};
use serde_json::json;

use super::backends::{self, hint_for};
use super::names::{is_self, self_error};
use super::Ctx;
use crate::ui::Failure;

/// Resolves every spec to one package. With `ask`, a name that matches several prompts on a terminal.
pub fn resolve_specs(
    ctx: &Ctx,
    manifest: &Manifest,
    specs: &[String],
    ask: bool,
) -> Result<Vec<Candidate>> {
    let flatpak = backends::flatpak();
    let nix = backends::nix()?;
    let remote = OnceCell::new();
    let catalog = |name: &str| Ok(manifest.get(name).cloned());
    let search = |query: &str| {
        let f = flatpak.as_ref().ok_or(FlathubError::NotAvailable)?;
        remote.get_or_init(|| f.ensure_remote().is_ok());
        f.search(query)
    };
    let exists = |attr: &str| match &nix {
        Some(n) => n.exists(attr),
        None => Err(NixError::NotAvailable("nix".into())),
    };
    let rctx = Context {
        catalog: &catalog,
        flathub_search: &search,
        nix_exists: &exists,
        check_nix: true,
    };
    let mut out: Vec<Candidate> = Vec::new();
    for spec in specs {
        let chosen = resolve_one(ctx, &rctx, spec, ask)?;
        if !out.contains(&chosen) {
            out.push(chosen);
        }
    }
    Ok(out)
}

fn resolve_one(ctx: &Ctx, rctx: &Context, spec: &str, ask: bool) -> Result<Candidate> {
    if is_self(spec) {
        return Err(self_error());
    }
    let outcome = resolve(spec, rctx)?;
    match outcome.resolution {
        Resolution::One(c) => Ok(c),
        Resolution::Many(cs) => choose_one(ctx, spec, cs, ask),
        Resolution::None => Err(not_found(spec, &outcome.skipped).into()),
    }
}

fn choose_one(
    ctx: &Ctx,
    spec: &str,
    mut candidates: Vec<Candidate>,
    ask: bool,
) -> Result<Candidate> {
    let labels: Vec<String> = candidates.iter().map(label).collect();
    if !ask || !ctx.ui.interactive() {
        let mut hints = labels;
        hints.push("pick one and pass it as backend:id".into());
        return Err(Failure {
            extra: Some(json!({ "spec": spec, "candidates": candidates })),
            ..Failure::new(format!("`{spec}` matches several packages"), hints)
        }
        .into());
    }
    let index = ctx
        .ui
        .choose(&format!("`{spec}` matches several packages"), &labels)?;
    Ok(candidates.swap_remove(index))
}

fn label(c: &Candidate) -> String {
    if c.name == c.id {
        format!("{}:{}", c.source, c.id)
    } else {
        format!("{}:{}  {}", c.source, c.id, c.name)
    }
}

fn not_found(spec: &str, skipped: &[Source]) -> Failure {
    let mut hints: Vec<String> = skipped
        .iter()
        .map(|s| match s {
            Source::Nix => format!("nixpkgs was not searched: {}", hint_for(*s)),
            _ => format!("Flathub was not searched: {}", hint_for(*s)),
        })
        .collect();
    if hints.is_empty() {
        hints.push("run `kit add` with no arguments to browse the Facile catalog".into());
    }
    let names: Vec<&str> = skipped.iter().map(|s| s.as_str()).collect();
    Failure {
        extra: Some(json!({ "spec": spec, "skipped": names })),
        ..Failure::new(format!("no package named `{spec}`"), hints)
    }
}

/// The catalog picker behind a bare `kit add`. Listed and installed tools start checked.
pub fn pick_catalog(ctx: &Ctx, manifest: &Manifest, listed: &[String]) -> Result<Vec<String>> {
    if !ctx.ui.interactive() {
        bail!("no package named: pass names or --all when not on a terminal");
    }
    let width = manifest
        .tools
        .iter()
        .map(|t| t.name.len())
        .max()
        .unwrap_or(0);
    let label = |t: &Tool| format!("{:<width$}  {}", t.name, ctx.ui.dim(&t.summary));
    let items: Vec<String> = manifest.tools.iter().map(label).collect();
    let checked: Vec<bool> = manifest
        .tools
        .iter()
        .map(|t| wanted(ctx, t, listed))
        .collect();
    let chosen = ctx.ui.pick(
        "Facile Studio (space to toggle, enter to add)",
        &items,
        &checked,
    )?;
    let names = chosen
        .into_iter()
        .map(|i| format!("facile:{}", manifest.tools[i].name));
    Ok(names.collect())
}

fn wanted(ctx: &Ctx, tool: &Tool, listed: &[String]) -> bool {
    listed.iter().any(|l| l.eq_ignore_ascii_case(&tool.name))
        || installed_tool(tool, &ctx.bin_dir).is_some()
}
