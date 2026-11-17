use std::path::Path;

use super::fetch::Fetch;
use super::lists::fetch_list;
use super::toml_spec::{facile_toml_probe, mise_toml_probe, parse_facile_toml, parse_mise_toml};
use super::{CatalogError, SingleSource, SourceConfig, Tool};

/// Tools found in every repo the configured list sources name, plus the errors met on the way.
pub(crate) fn discover_from_lists(
    fetch: &dyn Fetch,
    cache_dir: &Path,
    cfg: &SourceConfig,
) -> (Vec<Tool>, Vec<String>) {
    let mut tools = Vec::new();
    let mut errors = Vec::new();
    for ls in &cfg.lists {
        let repos = match fetch_list(fetch, cache_dir, &ls.url) {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("list source {:?} ({}): {e}", ls.name, ls.url));
                continue;
            }
        };
        for repo in repos {
            match discover_tool(fetch, &repo, "") {
                Ok(Some(t)) => tools.push(t),
                Ok(None) => {}
                Err(e) => errors.push(format!("discovering {repo} from list {:?}: {e}", ls.name)),
            }
        }
    }
    (tools, errors)
}

/// Turns one user entry into a tool, reading the repo's facile.toml when the entry is incomplete.
pub(crate) fn resolve_single(fetch: &dyn Fetch, s: &SingleSource) -> Result<Tool, CatalogError> {
    let incomplete =
        s.name.is_empty() || (s.build.is_empty() && s.bin.is_empty() && s.bins.is_empty());
    if s.repo.is_empty() || !incomplete {
        return s.to_tool();
    }
    match discover_tool(fetch, &s.repo, &s.branch) {
        Ok(Some(mut tool)) => {
            s.apply_to(&mut tool);
            Ok(tool)
        }
        Ok(None) if s.name.is_empty() => Err(CatalogError::Invalid(format!(
            "no facile.toml or [facile] block found in {}",
            s.repo
        ))),
        Ok(None) => s.to_tool(),
        Err(e) => Err(CatalogError::Invalid(format!(
            "discovering {}: {e}",
            s.repo
        ))),
    }
}

/// Reads facile.toml, then the `[facile]` block of mise.toml, from the repo's branches.
pub(crate) fn discover_tool(
    fetch: &dyn Fetch,
    repo: &str,
    branch: &str,
) -> Result<Option<Tool>, CatalogError> {
    let first = probe_repo_file(fetch, repo, "facile.toml", branch);
    if let Ok(Some(t)) = first {
        return Ok(Some(t));
    }
    match probe_repo_file(fetch, repo, "mise.toml", branch) {
        Ok(Some(t)) => Ok(Some(t)),
        Err(e) => Err(e),
        Ok(None) => first,
    }
}

fn probe_repo_file(
    fetch: &dyn Fetch,
    repo: &str,
    file: &str,
    branch: &str,
) -> Result<Option<Tool>, CatalogError> {
    let mut branches: Vec<&str> = Vec::new();
    for b in [branch, "main", "master"] {
        if !b.is_empty() && !branches.contains(&b) {
            branches.push(b);
        }
    }
    let mut last_error = None;
    for b in branches {
        match probe_branch(fetch, repo, file, b) {
            Ok(Some(tool)) => return Ok(Some(tool)),
            Ok(None) => {}
            Err(CatalogError::Fetch(e)) if e.is_not_found() => {}
            Err(e) => last_error = Some(e),
        }
    }
    last_error.map_or(Ok(None), Err)
}

fn probe_branch(
    fetch: &dyn Fetch,
    repo: &str,
    file: &str,
    branch: &str,
) -> Result<Option<Tool>, CatalogError> {
    let raw = fetch.get(&format!(
        "https://raw.githubusercontent.com/{repo}/{branch}/{file}"
    ))?;
    let facile = file == "facile.toml";
    let found = if facile {
        facile_toml_probe(&raw)
    } else {
        mise_toml_probe(&raw)
    };
    if !found {
        return Ok(None);
    }
    let parsed = if facile {
        parse_facile_toml(&raw)
    } else {
        parse_mise_toml(&raw)
    };
    match parsed {
        Ok(spec) => spec.to_tool().map(Some),
        Err(CatalogError::NoFacileToml) => Ok(None),
        Err(e) => Err(CatalogError::Invalid(format!(
            "parsing {repo}/{file}@{branch}: {e}"
        ))),
    }
}
