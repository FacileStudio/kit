use anyhow::Result;
use kit_core::listfile::Source;
use serde_json::json;

use super::apply::{self, Done, State};
use super::check::measure;
use super::del::{remove_flathub, uninstall_tool};
use super::drift::Item;
use super::Ctx;

/// `kit clean`: remove what kit can see installed but the list does not name.
///
/// Facile tools are removed only from the bin directory and only when kit placed them. Nix is touched only
/// inside kit's own profile, by rebuilding it from the list.
pub fn run(ctx: &Ctx, yes: bool) -> Result<()> {
    let extra = measure(ctx, false)?.extra;
    if extra.is_empty() {
        if ctx.ui.json() {
            ctx.ui.emit(&json!({ "file": ctx.file, "removed": [] }));
        }
        ctx.ui.success("Nothing to clean");
        return Ok(());
    }
    extra
        .iter()
        .for_each(|i| ctx.ui.out(&format!("- {}:{}", i.source, i.id)));
    if !ctx
        .ui
        .confirm(&format!("Remove {} package(s)?", extra.len()), yes)?
    {
        ctx.ui.step("Nothing changed");
        return Ok(());
    }
    let done = remove(ctx, &extra)?;
    if ctx.ui.json() {
        ctx.ui.emit(&json!({ "file": ctx.file, "removed": done }));
    }
    apply::finish(ctx, &done)
}

fn remove(ctx: &Ctx, extra: &[Item]) -> Result<Vec<Done>> {
    let ids = |source: Source| -> Vec<String> {
        extra
            .iter()
            .filter(|i| i.source == source.as_str())
            .map(|i| i.id.clone())
            .collect()
    };
    let manifest = ctx.catalog()?;
    let mut done: Vec<Done> = ids(Source::Facile)
        .iter()
        .filter_map(|name| manifest.get(name))
        .map(|tool| uninstall_tool(ctx, tool))
        .collect();
    done.extend(remove_flathub(ctx, &ids(Source::Flathub)));
    let nix = ids(Source::Nix);
    if !nix.is_empty() {
        done.extend(apply::sync_nix(
            ctx,
            &ctx.list()?.nix,
            &nix,
            State::Removed,
        )?);
    }
    Ok(done)
}
