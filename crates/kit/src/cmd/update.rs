use anyhow::Result;
use kit_core::catalog::Manifest;
use kit_core::facile::{installed_tool, outdated, UpdateOptions};
use kit_core::listfile::Source;
use serde_json::json;

use super::apply::{self, Done, State};
use super::drift::Item;
use super::names::split_self;
use super::selfupdate::update_self;
use super::update_parts::{self, targets, Targets};
use super::Ctx;
use crate::cli::{SelfArgs, UpdateArgs};

/// `kit update`: Facile tools, Flathub apps, the Nix profile, then kit itself last.
pub fn run(ctx: &Ctx, args: &UpdateArgs) -> Result<()> {
    let (want_self, rest, named) = split_self(&args.names);
    let file = ctx.list()?;
    let manifest = ctx.catalog()?;
    let todo = targets(&file, &manifest, &rest, named)?;
    if args.check {
        return check(ctx, &todo);
    }
    let mut done = facile(ctx, &manifest, &todo, args)?;
    if todo.flathub {
        done.extend(update_parts::flathub(ctx, file.flathub.len()));
    }
    if todo.nix {
        done.extend(update_parts::nix(ctx, &file.nix, args.force)?);
    }
    if want_self {
        done.extend(self_step(ctx, args));
    }
    if ctx.ui.json() {
        ctx.ui.emit(&json!({ "updated": done }));
    }
    apply::finish(ctx, &done)
}

fn self_step(ctx: &Ctx, args: &UpdateArgs) -> Option<Done> {
    let opts = SelfArgs {
        force: args.force,
        source: args.source,
    };
    let err = update_self(ctx, opts).err()?;
    ctx.ui.error(&format!("{err:#}"));
    Some(Done::new(
        Source::Facile,
        "kit",
        State::Failed,
        err.to_string(),
    ))
}

fn facile(ctx: &Ctx, manifest: &Manifest, todo: &Targets, args: &UpdateArgs) -> Result<Vec<Done>> {
    if todo.facile.is_empty() {
        return Ok(Vec::new());
    }
    let opts = UpdateOptions {
        force: args.force,
        source: args.source,
        no_skill: args.no_skill,
        bin_dir: ctx.bin_dir.clone(),
        mcp_tools: manifest.mcp.clone(),
        ..UpdateOptions::default()
    };
    let done = update_parts::facile(ctx, &ctx.backend()?, &todo.facile, &opts);
    apply::report_path(ctx);
    Ok(done)
}

fn check(ctx: &Ctx, todo: &Targets) -> Result<()> {
    let stale = stale(ctx, todo)?;
    if ctx.ui.json() {
        ctx.ui.emit(&json!({ "outdated": stale }));
        return Ok(());
    }
    for item in &stale {
        ctx.ui.out(&format!(
            "~ facile:{}  {} → {}",
            item.id, item.version, item.latest
        ));
    }
    match stale.len() {
        0 => ctx.ui.success("Every listed Facile tool is up to date"),
        n => ctx.ui.out(
            &ctx.ui
                .dim(&format!("{n} update(s) available, run `kit update`")),
        ),
    }
    Ok(())
}

fn stale(ctx: &Ctx, todo: &Targets) -> Result<Vec<Item>> {
    let backend = ctx.backend()?;
    let repos: Vec<String> = todo.facile.iter().map(|t| t.repo.clone()).collect();
    let latest = backend.github.latest(&repos);
    let mut out = Vec::new();
    for tool in &todo.facile {
        let Some(have) = installed_tool(tool, &ctx.bin_dir) else {
            continue;
        };
        let newest = latest
            .get(&tool.repo)
            .map(|t| t.trim_start_matches('v'))
            .unwrap_or_default();
        if outdated(&have.version, newest) {
            let latest = newest.to_string();
            out.push(Item {
                source: "facile",
                id: tool.name.clone(),
                version: have.version,
                latest,
            });
        }
    }
    Ok(out)
}
