use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::cache;
use super::fetch::{Fetch, FetchError};

const EMPTY_MARK: &str = "# empty";

fn dedupe(items: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(items.len());
    for item in items {
        if !out.contains(&item) {
            out.push(item);
        }
    }
    out
}

/// Extracts repo identifiers from a list body: PEP 503 `<a href="/owner/repo/">` links when there are any,
/// else one `owner/repo` per line, skipping comments, blank lines and anything without exactly one slash.
pub fn parse_list_source(body: &str) -> Vec<String> {
    let links: Vec<String> = regex::Regex::new(r#"<a href="/([^/]+/[^/]+)/?">"#)
        .map(|re| re.captures_iter(body).map(|c| c[1].to_string()).collect())
        .unwrap_or_default();
    if !links.is_empty() {
        return dedupe(links);
    }
    let lines = body
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#') && l.matches('/').count() == 1)
        .map(String::from)
        .collect();
    dedupe(lines)
}

/// Where a list URL's repos are cached: `lists/<sha256 of the URL>` under the cache directory.
pub(crate) fn cache_path(cache_dir: &Path, url: &str) -> PathBuf {
    cache_dir
        .join("lists")
        .join(hex::encode(Sha256::digest(url.as_bytes())))
}

fn read_cached(path: &Path) -> Option<Vec<String>> {
    cache::younger_than(cache::mtime(path)?, cache::MAX_AGE)?;
    let raw = fs::read_to_string(path).ok()?;
    let content = raw.trim();
    if content == EMPTY_MARK {
        return Some(Vec::new());
    }
    if content.is_empty() {
        return None;
    }
    Some(
        content
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(String::from)
            .collect(),
    )
}

fn write_cached(path: &Path, repos: &[String]) {
    let content = if repos.is_empty() {
        EMPTY_MARK.to_string()
    } else {
        repos.join("\n")
    };
    cache::write(path, content.as_bytes());
}

/// The repos a list URL names, from a day-old cache when there is one.
pub(crate) fn fetch_list(
    fetch: &dyn Fetch,
    cache_dir: &Path,
    url: &str,
) -> Result<Vec<String>, FetchError> {
    let path = cache_path(cache_dir, url);
    if let Some(repos) = read_cached(&path) {
        return Ok(repos);
    }
    let body = fetch.get(url)?;
    let repos = parse_list_source(&String::from_utf8_lossy(&body));
    write_cached(&path, &repos);
    Ok(repos)
}
