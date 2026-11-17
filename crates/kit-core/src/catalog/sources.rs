use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use super::sources_parse::{parse_root_item, parse_tools_section};
use super::{CatalogError, SingleSource};
use crate::paths;

/// The user's own sources: explicit tools and URLs listing repos.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SourceConfig {
    /// Explicit tool entries.
    pub single: Vec<SingleSource>,
    /// URLs returning repo identifiers.
    pub lists: Vec<ListSource>,
}

/// A URL that returns repos, one `owner/repo` per line or as a PEP 503 simple index.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ListSource {
    /// A label for messages.
    pub name: String,
    /// Where the list lives.
    pub url: String,
}

/// The sources file candidates under the real home and config directories.
pub fn sources_paths() -> Vec<PathBuf> {
    match (paths::home(), paths::config_dir()) {
        (Ok(home), Ok(config)) => sources_paths_in(&home, &config),
        _ => Vec::new(),
    }
}

/// The sources file candidates in priority order: kit's own config, then facile's legacy files.
///
/// `kit_config` is kit's config directory; facile's legacy directory is its sibling `facile`.
pub fn sources_paths_in(home: &Path, kit_config: &Path) -> Vec<PathBuf> {
    let legacy = kit_config.parent().unwrap_or(kit_config).join("facile");
    vec![
        kit_config.join("config.yml"),
        kit_config.join("config.yaml"),
        home.join(".facile.yml"),
        home.join(".facile.yaml"),
        legacy.join("config.yml"),
        legacy.join("config.yaml"),
        legacy.join("sources.yml"),
        legacy.join("sources.yaml"),
    ]
}

/// The first candidate that exists.
pub fn sources_path(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates.iter().find(|p| p.exists()).cloned()
}

/// Reads and parses the first candidate that exists. No file is an empty config, not an error.
pub(crate) fn load_from(candidates: &[PathBuf]) -> Result<SourceConfig, CatalogError> {
    let Some(path) = sources_path(candidates) else {
        return Ok(SourceConfig::default());
    };
    let raw = fs::read_to_string(&path).map_err(|source| CatalogError::Read {
        path: path.clone(),
        source,
    })?;
    parse_sources(&raw)
        .map_err(|e| CatalogError::Invalid(format!("parsing sources file {}: {e}", path.display())))
}

/// Parses a sources file: a root map of tools, `tools:`/`single:`/`sources:` blocks, `lists:`, or a bare list.
pub fn parse_sources(raw: &str) -> Result<SourceConfig, CatalogError> {
    let mut cfg = SourceConfig::default();
    if raw.trim().is_empty() {
        return Ok(cfg);
    }
    let doc: Value = serde_yaml::from_str(raw)
        .map_err(|e| CatalogError::Invalid(format!("parsing sources: {e}")))?;
    match doc {
        Value::Null => {}
        Value::Sequence(_) => cfg.single = parse_tools_section(&doc)?,
        Value::Mapping(map) => {
            for (key, val) in &map {
                parse_root_item(&mut cfg, key, val)?;
            }
        }
        _ => {
            return Err(CatalogError::Invalid(
                "parsing sources: expected a map or a list".into(),
            ))
        }
    }
    Ok(cfg)
}
