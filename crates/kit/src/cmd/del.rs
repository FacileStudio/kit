use anyhow::Result;
use kit_core::catalog::{Manifest, Tool};
use kit_core::facile::{owner_of, uninstall};
use kit_core::listfile::{KitFile, Source};
use serde_json::json;

use super::apply::{self, Done, State};
use super::backends;
use super::names::{is_self, self_error, split_names};
use super::Ctx;
use crate::cli::DelArgs;
use crate::ui::Failure;

/// `kit del`: drop entries from the list file, then uninstall them.
pub fn run(ctx: &Ctx, args: &DelArgs) -> Result<()> {
    let manifest = ctx.catalog()?;
    let targets = targets(&ctx.list()?, &manifest, &args.specs)?;
    let nix_left = ctx.edit(|f| {
        targets.iter().for_each(|(s, id)| {
            f.remove(*s, id);
        });
        f.nix.clone()
    })?;
    let ids = |source: Source| -> Vec<String> {
        targets
            .iter()
            .filter(|(s, _)| *s == source)
            .map(|(_, id)| id.clone())
            .collect()
    };
    let mut done: Vec<Done> = ids(Source::Facile)
        .iter()
        .map(|id| remove_facile(ctx, &manifest, id))
        .collect();
    done.extend(remove_flathub(ctx, &ids(Source::Flathub)));
    let nix = ids(Source::Nix);
    if !nix.is_empty() {
        done.extend(apply::sync_nix(ctx, &nix_left, &nix, State::Removed)?);
    }
    if ctx.ui.json() {
        ctx.ui.emit(&json!({ "file": ctx.file, "removed": done }));
    }
    apply::finish(ctx, &done)
}

fn targets(file: &KitFile, manifest: &Manifest, specs: &[String]) -> Result<Vec<(Source, String)>> {
    let mut out: Vec<(Source, String)> = Vec::new();
    for spec in split_names(specs) {
        let found = find(file, manifest, &spec)?;
        if !out.contains(&found) {
            out.push(found);
        }
    }
    Ok(out)
}

/// The list entry a spec names: exact `backend:id`, or a bare name searched in every section.
///
/// A Facile tool that is not listed still resolves, so `kit del` removes what an older facile installed.
pub fn find(file: &KitFile, manifest: &Manifest, spec: &str) -> Result<(Source, String)> {
    if is_self(spec) {
        return Err(self_error());
    }
    let (source, name) = Source::split_spec(spec)?;
    let sources = source.map_or(Source::ALL.to_vec(), |s| vec![s]);
    let hits: Vec<(Source, String)> = sources
        .iter()
        .flat_map(|s| {
            file.entries(*s)
                .iter()
                .filter(|e| matches(*s, e, name))
                .map(|e| (*s, e.clone()))
        })
        .collect();
    match hits.len() {
        1 => Ok(hits[0].clone()),
        0 => unlisted(manifest, source, name, spec),
        _ => {
            let mut hints: Vec<String> = hits.iter().map(|(s, id)| format!("{s}:{id}")).collect();
            hints.push("pick one and pass it as backend:id".into());
            Err(Failure::new(format!("`{spec}` matches several listed packages"), hints).into())
        }
    }
}

fn unlisted(
    manifest: &Manifest,
    source: Option<Source>,
    name: &str,
    spec: &str,
) -> Result<(Source, String)> {
    match manifest
        .get(name)
        .filter(|_| source.is_none_or(|s| s == Source::Facile))
    {
        Some(tool) => Ok((Source::Facile, tool.name.clone())),
        None => Err(Failure::new(
            format!("`{spec}` is not in the list file"),
            vec!["run `kit list` to see what is listed".into()],
        )
        .into()),
    }
}

fn matches(source: Source, entry: &str, name: &str) -> bool {
    entry.eq_ignore_ascii_case(name)
        || (source == Source::Flathub
            && entry
                .rsplit('.')
                .next()
                .is_some_and(|last| last.eq_ignore_ascii_case(name)))
}

fn remove_facile(ctx: &Ctx, manifest: &Manifest, name: &str) -> Done {
    let Some(tool) = manifest.get(name) else {
        ctx.ui.warn(&format!(
            "{name} is not in the catalog, only the list entry was removed"
        ));
        return Done::new(Source::Facile, name, State::Removed, "");
    };
    let owner = owner_of(&ctx.bin_dir.join(&tool.bin), &ctx.bin_dir);
    if owner.is_foreign() {
        ctx.ui.warn(&format!(
            "{name} is managed by {owner}, kit leaves the binary alone"
        ));
        return Done::new(
            Source::Facile,
            name,
            State::Removed,
            format!("managed by {owner}"),
        );
    }
    uninstall_tool(ctx, tool)
}

/// Removes a Facile tool's binaries from the bin directory and reports it.
pub fn uninstall_tool(ctx: &Ctx, tool: &Tool) -> Done {
    let name = &tool.name;
    let (state, detail) = match uninstall(&ctx.bin_dir, tool) {
        Ok(true) => {
            ctx.ui.success(&format!("{name} removed"));
            (State::Removed, String::new())
        }
        Ok(false) => {
            ctx.ui.warn(&format!(
                "{name} is not installed in {}",
                ctx.tilde(&ctx.bin_dir)
            ));
            (State::Removed, "not installed".to_string())
        }
        Err(e) => {
            ctx.ui.error(&e.to_string());
            (State::Failed, e.to_string())
        }
    };
    Done::new(Source::Facile, name, state, detail)
}

/// Uninstalls the Flathub apps among `ids` that are installed.
pub fn remove_flathub(ctx: &Ctx, ids: &[String]) -> Vec<Done> {
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
    let installed = flatpak.list_installed().unwrap_or_default();
    let refs: Vec<&str> = ids
        .iter()
        .filter(|id| installed.contains(id))
        .map(String::as_str)
        .collect();
    match flatpak.uninstall(&refs) {
        Ok(()) => {
            ctx.ui.success(&format!("{} removed", ids.join(", ")));
            mark(State::Removed, "")
        }
        Err(e) => {
            ctx.ui.error(&e.to_string());
            mark(State::Failed, &e.to_string())
        }
    }
}
