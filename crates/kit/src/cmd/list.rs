use std::collections::HashMap;

use anyhow::Result;
use kit_core::catalog::Manifest;
use kit_core::facile::{outdated, Backend, KIT_REPO};
use kit_core::listfile::{KitFile, Source};
use serde::Serialize;

use super::list_render::render;
use super::machine::{probe, Machine};
use super::{Ctx, VERSION};
use crate::cli::ListArgs;

type Tags = HashMap<String, String>;

/// One listing row.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Row {
    /// The list file ID, or `kit` for kit itself.
    pub name: String,
    /// Whether it is on this machine.
    pub installed: bool,
    /// The installed version, when the backend reports one.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub version: String,
    /// The latest release, when known.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub latest: String,
    /// True only when the installed version orders strictly before the latest.
    pub outdated: bool,
    /// What the package is.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub summary: String,
    /// Why the state is unknown.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub note: String,
}

/// The listing, one section per source.
#[derive(Debug, Default, Serialize)]
pub struct Listing {
    /// kit, then the listed Facile tools.
    pub facile: Vec<Row>,
    /// The listed Nix attributes.
    pub nix: Vec<Row>,
    /// The listed Flathub apps.
    pub flathub: Vec<Row>,
}

/// `kit list`: what the list file names and what the machine has.
pub fn run(ctx: &Ctx, args: &ListArgs) -> Result<()> {
    let file = ctx.list()?;
    if ctx.ui.quiet() && !ctx.ui.json() {
        let ids = Source::ALL.iter().flat_map(|s| file.entries(*s));
        ids.for_each(|id| println!("{id}"));
        return Ok(());
    }
    let manifest = ctx.catalog()?;
    let backend = ctx.backend()?;
    let machine = probe(ctx, &backend, &manifest)?;
    let latest = tags(&backend, &manifest, &machine, args.check);
    let listing = build(&file, &machine, &manifest, &latest);
    if ctx.ui.json() {
        ctx.ui.emit(&listing);
    } else {
        render(ctx, &listing);
    }
    Ok(())
}

fn tags(backend: &Backend, manifest: &Manifest, machine: &Machine, live: bool) -> Tags {
    let mut repos = vec![KIT_REPO.to_string()];
    let installed = machine.facile.iter().filter_map(|i| manifest.get(&i.name));
    repos.extend(installed.map(|t| t.repo.clone()));
    if live {
        return backend.github.latest(&repos);
    }
    let now = std::time::SystemTime::now();
    backend.latest_cache().resolve(&backend.github, &repos, now)
}

fn tag(latest: &Tags, repo: &str) -> String {
    let found = latest.get(repo).map(|t| t.trim_start_matches('v'));
    found.unwrap_or_default().to_string()
}

fn build(file: &KitFile, machine: &Machine, manifest: &Manifest, latest: &Tags) -> Listing {
    let kit_latest = tag(latest, KIT_REPO);
    let kit = Row {
        name: "kit".into(),
        installed: true,
        version: VERSION.into(),
        outdated: outdated(VERSION, &kit_latest),
        latest: kit_latest,
        summary: "The Facile package manager".into(),
        note: String::new(),
    };
    let tools = file
        .facile
        .iter()
        .map(|n| facile_row(n, machine, manifest, latest));
    let flathub = machine.flathub.clone().unwrap_or_default();
    let flathub_note = note(machine.flathub.is_none(), "flatpak is not installed");
    let nix_note = note(!machine.nix_available, "nix is not installed");
    Listing {
        facile: std::iter::once(kit).chain(tools).collect(),
        nix: plain(&file.nix, &machine.nix, &nix_note),
        flathub: plain(&file.flathub, &flathub, &flathub_note),
    }
}

fn facile_row(name: &str, machine: &Machine, manifest: &Manifest, latest: &Tags) -> Row {
    let tool = manifest.get(name);
    let found = machine
        .facile
        .iter()
        .find(|i| i.name.eq_ignore_ascii_case(name));
    let version = found.map(|i| i.version.clone()).unwrap_or_default();
    let newest = match (found, tool) {
        (Some(_), Some(t)) => tag(latest, &t.repo),
        _ => String::new(),
    };
    Row {
        name: name.to_string(),
        installed: found.is_some(),
        outdated: outdated(&version, &newest),
        version,
        latest: newest,
        summary: tool.map(|t| t.summary.clone()).unwrap_or_default(),
        note: note(tool.is_none(), "not in the catalog"),
    }
}

fn note(when: bool, text: &str) -> String {
    if when {
        text.to_string()
    } else {
        String::new()
    }
}

fn plain(listed: &[String], have: &[String], why: &str) -> Vec<Row> {
    let row = |id: &String| Row {
        name: id.clone(),
        installed: have.contains(id),
        note: note(!have.contains(id), why),
        ..Row::default()
    };
    listed.iter().map(row).collect()
}
