use serde_yaml::Value;

use super::sources_parse::invalid;
use super::{CatalogError, ListSource, SingleSource};

/// Parses `lists:`, whose items are bare URLs or `{name, url}` objects.
pub(crate) fn parse_lists(val: &Value) -> Result<Vec<ListSource>, CatalogError> {
    let Value::Sequence(seq) = val else {
        return Err(invalid("lists must be an array".into()));
    };
    seq.iter()
        .map(|item| match item {
            Value::String(url) => Ok(ListSource {
                name: list_name(url),
                url: url.clone(),
            }),
            other => serde_yaml::from_value(other.clone()).map_err(|e| invalid(e.to_string())),
        })
        .collect()
}

fn list_name(url: &str) -> String {
    let base = url.trim_end_matches('/').rsplit('/').next().unwrap_or("");
    let stem = match base.rfind('.') {
        Some(i) => &base[..i],
        None => base,
    };
    if stem.is_empty() {
        url.to_string()
    } else {
        stem.to_string()
    }
}

/// Parses one tool: `name: owner/repo[@branch]`, a bare `owner/repo`, or a full object.
pub(crate) fn parse_tool_entry(name: &str, val: &Value) -> Result<SingleSource, CatalogError> {
    match val {
        Value::Null => Ok(shorthand(name, "")),
        Value::String(s) => Ok(shorthand(name, s)),
        other => {
            let mut s: SingleSource =
                serde_yaml::from_value(other.clone()).map_err(|e| invalid(e.to_string()))?;
            if s.name.is_empty() {
                s.name = name.to_string();
            }
            Ok(s)
        }
    }
}

fn shorthand(name: &str, repo_text: &str) -> SingleSource {
    let (mut repo, mut branch) = split_at(repo_text);
    let mut name = name.to_string();
    if branch.is_none() && name.contains('@') {
        let (n, b) = split_at(&name);
        (name, branch) = (n, b);
    }
    if repo.is_empty() && name.contains('/') {
        repo = name.clone();
        name = base(&repo);
    }
    if name.is_empty() && !repo.is_empty() {
        name = base(&repo);
    }
    SingleSource {
        name,
        repo,
        branch: branch.unwrap_or_default(),
        ..SingleSource::default()
    }
}

fn split_at(text: &str) -> (String, Option<String>) {
    match text.split_once('@') {
        Some((a, b)) => (a.to_string(), Some(b.to_string())),
        None => (text.to_string(), None),
    }
}

fn base(path: &str) -> String {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path)
        .to_string()
}
