use std::collections::HashMap;

use kit_core::catalog::Manifest;
use kit_core::facile::{outdated, InstalledTool, Owner};
use kit_core::listfile::{KitFile, Source};
use serde::Serialize;

use super::machine::Machine;

/// One package in a drift report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    /// The backend name.
    pub source: &'static str,
    /// The list file ID.
    pub id: String,
    /// The installed version, when known.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub version: String,
    /// The latest version, for outdated items.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub latest: String,
}

/// The difference between the list file and the machine.
#[derive(Debug, Default, Serialize)]
pub struct Drift {
    /// Listed but not installed.
    pub missing: Vec<Item>,
    /// Installed but not listed.
    pub extra: Vec<Item>,
    /// Listed and installed, with a newer release.
    pub outdated: Vec<Item>,
    /// Backends with listed entries that are not on this machine.
    pub skipped: Vec<&'static str>,
}

impl Drift {
    /// True when the machine matches the file.
    pub fn is_clean(&self) -> bool {
        self.missing.is_empty() && self.extra.is_empty() && self.outdated.is_empty()
    }
}

fn item(source: Source, id: &str) -> Item {
    Item {
        source: source.as_str(),
        id: id.to_string(),
        version: String::new(),
        latest: String::new(),
    }
}

/// Compares the list file with the machine. `latest` maps repos to tags for the outdated check.
pub fn compute(
    file: &KitFile,
    machine: &Machine,
    manifest: &Manifest,
    latest: &HashMap<String, String>,
) -> Drift {
    let mut drift = Drift::default();
    facile(&mut drift, file, machine, manifest, latest);
    let flathub = machine.flathub.clone().unwrap_or_default();
    if machine.flathub.is_none() && !file.flathub.is_empty() {
        drift.skipped.push(Source::Flathub.as_str());
    }
    if !machine.nix_available && !file.nix.is_empty() {
        drift.skipped.push(Source::Nix.as_str());
    }
    sets(&mut drift, Source::Flathub, &file.flathub, &flathub);
    sets(&mut drift, Source::Nix, &file.nix, &machine.nix);
    drift
}

fn sets(drift: &mut Drift, source: Source, listed: &[String], have: &[String]) {
    drift.missing.extend(
        listed
            .iter()
            .filter(|id| !have.contains(id))
            .map(|id| item(source, id)),
    );
    drift.extra.extend(
        have.iter()
            .filter(|id| !listed.contains(id))
            .map(|id| item(source, id)),
    );
}

fn facile(
    drift: &mut Drift,
    file: &KitFile,
    machine: &Machine,
    manifest: &Manifest,
    latest: &HashMap<String, String>,
) {
    for name in &file.facile {
        if !machine
            .facile
            .iter()
            .any(|i| i.name.eq_ignore_ascii_case(name))
        {
            drift.missing.push(item(Source::Facile, name));
        }
    }
    for tool in &machine.facile {
        if file
            .facile
            .iter()
            .any(|l| l.eq_ignore_ascii_case(&tool.name))
        {
            drift.outdated.extend(outdated_item(tool, manifest, latest));
        } else if tool.owner == Owner::Kit {
            drift.extra.push(Item {
                version: tool.version.clone(),
                ..item(Source::Facile, &tool.name)
            });
        }
    }
}

fn outdated_item(
    tool: &InstalledTool,
    manifest: &Manifest,
    latest: &HashMap<String, String>,
) -> Option<Item> {
    let repo = manifest
        .get(&tool.name)
        .map(|t| t.repo.as_str())
        .unwrap_or_default();
    let newest = latest
        .get(repo)
        .map(|t| t.trim_start_matches('v').to_string())
        .unwrap_or_default();
    outdated(&tool.version, &newest).then(|| Item {
        version: tool.version.clone(),
        latest: newest,
        ..item(Source::Facile, &tool.name)
    })
}
