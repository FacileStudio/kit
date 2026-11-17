use serde_yaml::{Mapping, Value};

use super::sources_entry::{parse_lists, parse_tool_entry};
use super::{CatalogError, SingleSource, SourceConfig};

pub(crate) fn invalid(msg: String) -> CatalogError {
    CatalogError::Invalid(msg)
}

/// The key as text, whatever YAML scalar it was written as.
pub(crate) fn key_text(key: &Value) -> String {
    match key {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => serde_yaml::to_string(other)
            .map(|s| s.trim().to_string())
            .unwrap_or_default(),
    }
}

/// Parses one top-level entry of a sources file into `cfg`.
pub(crate) fn parse_root_item(
    cfg: &mut SourceConfig,
    key: &Value,
    val: &Value,
) -> Result<(), CatalogError> {
    let key = key_text(key);
    match key.to_lowercase().as_str() {
        "single" | "tools" => {
            let tools =
                parse_tools_section(val).map_err(|e| invalid(format!("parsing {key}: {e}")))?;
            cfg.single.extend(tools);
        }
        "sources" => parse_sources_block(cfg, val)?,
        "lists" => {
            let lists =
                parse_lists(val).map_err(|e| invalid(format!("parsing list sources: {e}")))?;
            cfg.lists.extend(lists);
        }
        _ => {
            let tool = parse_tool_entry(&key, val)
                .map_err(|e| invalid(format!("parsing tool {key:?}: {e}")))?;
            cfg.single.push(tool);
        }
    }
    Ok(())
}

fn parse_sources_block(cfg: &mut SourceConfig, val: &Value) -> Result<(), CatalogError> {
    let sections = ["tools", "single", "lists", "sources"];
    if let Value::Mapping(map) = val {
        if map
            .keys()
            .any(|k| sections.contains(&key_text(k).to_lowercase().as_str()))
        {
            for (k, v) in map {
                parse_root_item(cfg, k, v)?;
            }
            return Ok(());
        }
    }
    if !matches!(val, Value::Mapping(_) | Value::Sequence(_)) {
        return Err(invalid("sources must be a map or a list".into()));
    }
    let tools = parse_tools_section(val).map_err(|e| invalid(format!("parsing sources: {e}")))?;
    cfg.single.extend(tools);
    Ok(())
}

/// Parses a block of tools written either as a name-to-entry map or as a list.
pub(crate) fn parse_tools_section(val: &Value) -> Result<Vec<SingleSource>, CatalogError> {
    match val {
        Value::Mapping(map) => parse_tools_map(map),
        Value::Sequence(seq) => seq.iter().map(|item| parse_tool_entry("", item)).collect(),
        _ => Err(invalid("tools must be a map or a list".into())),
    }
}

fn parse_tools_map(map: &Mapping) -> Result<Vec<SingleSource>, CatalogError> {
    map.iter()
        .map(|(k, v)| parse_tool_entry(&key_text(k), v))
        .collect()
}
