//! Reading the list file from YAML and writing it back.

use std::io::ErrorKind;
use std::path::Path;

use serde::Deserialize;
use serde_yaml::{Mapping, Value};

use super::{KitFile, ListError, VERSION};

#[derive(Deserialize)]
struct Raw {
    version: Option<u64>,
    #[serde(default)]
    facile: Vec<String>,
    #[serde(default)]
    nix: Vec<String>,
    #[serde(default)]
    flathub: Vec<String>,
}

const KEYS: [&str; 4] = ["version", "facile", "nix", "flathub"];

/// Reads `path`, or `seed` when `path` is missing, or an empty list when both are.
pub(super) fn load_or_seed(path: &Path, seed: &Path) -> Result<KitFile, ListError> {
    for candidate in [path, seed] {
        if let Some(text) = read_text(candidate)? {
            return parse(&text, candidate);
        }
    }
    Ok(KitFile::default())
}

/// Parses list file YAML. `path` only labels errors.
pub(super) fn parse(text: &str, path: &Path) -> Result<KitFile, ListError> {
    let yaml = |source| ListError::Yaml {
        path: path.to_path_buf(),
        source,
    };
    let value: Value = serde_yaml::from_str(text).map_err(yaml)?;
    let map = match value {
        Value::Null => return Ok(KitFile::default()),
        Value::Mapping(map) => map,
        _ => {
            return Err(ListError::NotMapping {
                path: path.to_path_buf(),
            })
        }
    };
    check_keys(&map, path)?;
    let raw: Raw = serde_yaml::from_value(Value::Mapping(map)).map_err(yaml)?;
    let version = raw.version.unwrap_or(u64::from(VERSION));
    if version != u64::from(VERSION) {
        return Err(ListError::Version {
            path: path.to_path_buf(),
            found: version,
        });
    }
    Ok(KitFile {
        version: VERSION,
        facile: dedupe(raw.facile),
        nix: dedupe(raw.nix),
        flathub: dedupe(raw.flathub),
    })
}

/// Serializes `file` with each section deduplicated in first-seen order.
pub(super) fn render(file: &KitFile, path: &Path) -> Result<String, ListError> {
    let clean = KitFile {
        version: VERSION,
        facile: dedupe(file.facile.clone()),
        nix: dedupe(file.nix.clone()),
        flathub: dedupe(file.flathub.clone()),
    };
    serde_yaml::to_string(&clean).map_err(|source| ListError::Yaml {
        path: path.to_path_buf(),
        source,
    })
}

fn check_keys(map: &Mapping, path: &Path) -> Result<(), ListError> {
    for key in map.keys() {
        let name = match key {
            Value::String(name) => name.clone(),
            other => serde_yaml::to_string(other)
                .unwrap_or_default()
                .trim()
                .to_string(),
        };
        if !KEYS.contains(&name.as_str()) {
            return Err(ListError::UnknownKey {
                path: path.to_path_buf(),
                key: name,
            });
        }
    }
    Ok(())
}

fn dedupe(list: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(list.len());
    for entry in list {
        if !out.contains(&entry) {
            out.push(entry);
        }
    }
    out
}

fn read_text(path: &Path) -> Result<Option<String>, ListError> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(source) => Err(ListError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}
