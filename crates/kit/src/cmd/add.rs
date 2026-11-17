use anyhow::Result;
use kit_core::catalog::Manifest;
use kit_core::facile::InstallOptions;
use kit_core::listfile::Source;
use kit_core::resolve::Candidate;
use serde_json::json;

use super::apply::{self, Done, State};
use super::names::split_names;
use super::pick::{pick_catalog, resolve_specs};
use super::Ctx;
use crate::cli::AddArgs;

/// `kit add`: resolve, write the list file, then install exactly what was added.
pub fn run(ctx: &Ctx, args: &AddArgs) -> Result<()> {
    let manifest = ctx.catalog()?;
    let specs = specs(ctx, &manifest, args)?;
    if specs.is_empty() {
        ctx.ui.step("Nothing selected");
        return Ok(());
    }
    let chosen = resolve_specs(ctx, &manifest, &specs, !args.yes)?;
    let nix = ctx.edit(|file| {
        chosen.iter().for_each(|c| {
            file.add(c.source, &c.id);
        });
        file.nix.clone()
    })?;
    ctx.ui.debug(&format!(
        "listed {} package(s) in {}",
        chosen.len(),
        ctx.tilde(&ctx.file)
    ));
    let done = install(ctx, &manifest, &chosen, &nix, args)?;
    if ctx.ui.json() {
        ctx.ui.emit(&json!({ "file": ctx.file, "added": done }));
    }
    apply::finish(ctx, &done)
}

fn specs(ctx: &Ctx, manifest: &Manifest, args: &AddArgs) -> Result<Vec<String>> {
    if args.all {
        return Ok(manifest
            .tools
            .iter()
            .map(|t| format!("facile:{}", t.name))
            .collect());
    }
    if !args.specs.is_empty() {
        return Ok(split_names(&args.specs));
    }
    pick_catalog(ctx, manifest, &ctx.list()?.facile)
}

fn install(
    ctx: &Ctx,
    manifest: &Manifest,
    chosen: &[Candidate],
    nix: &[String],
    args: &AddArgs,
) -> Result<Vec<Done>> {
    let ids = |source: Source| -> Vec<String> {
        chosen
            .iter()
            .filter(|c| c.source == source)
            .map(|c| c.id.clone())
            .collect()
    };
    let mut done = Vec::new();
    let facile = ids(Source::Facile);
    if !facile.is_empty() {
        let backend = ctx.backend()?;
        let opts = apply::facile_opts(ctx, manifest, options(args));
        for tool in facile.iter().filter_map(|name| manifest.get(name)) {
            done.push(apply::install_facile(ctx, &backend, tool, &opts));
        }
        apply::report_path(ctx);
    }
    done.extend(apply::install_flathub(ctx, &ids(Source::Flathub)));
    let added_nix = ids(Source::Nix);
    if !added_nix.is_empty() {
        done.extend(apply::sync_nix(ctx, nix, &added_nix, State::Installed)?);
    }
    Ok(done)
}

fn options(args: &AddArgs) -> InstallOptions {
    InstallOptions {
        source: args.source,
        version: args.tag.clone().unwrap_or_default(),
        no_skill: args.no_skill,
        no_mcp: args.no_mcp,
        force: args.force,
        ..InstallOptions::default()
    }
}
