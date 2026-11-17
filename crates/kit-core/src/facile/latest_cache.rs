use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::github::Github;

/// How long a cached tag answers before kit asks GitHub again.
pub const LATEST_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Entry {
    tag: String,
    checked: u64,
}

/// The daily cache of latest release tags in `latest.json`, for listings that must not wait on GitHub.
///
/// `update` never reads it: the decision to download compares against the live tag.
#[derive(Debug, Clone)]
pub struct LatestCache {
    /// The cache file.
    pub path: PathBuf,
    /// How long an entry stays fresh.
    pub max_age: Duration,
}

fn now_secs(now: SystemTime) -> u64 {
    now.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl LatestCache {
    /// The cache at `<cache_dir>/latest.json` with the daily lifetime.
    pub fn in_dir(cache_dir: PathBuf) -> LatestCache {
        LatestCache {
            path: cache_dir.join("latest.json"),
            max_age: LATEST_MAX_AGE,
        }
    }

    /// Every repo mapped to its latest tag: fresh entries from the file, the rest resolved and saved.
    pub fn resolve(
        &self,
        github: &Github,
        repos: &[String],
        now: SystemTime,
    ) -> HashMap<String, String> {
        let mut entries = self.read();
        let at = now_secs(now);
        let fresh = |e: &Entry| at.saturating_sub(e.checked) < self.max_age.as_secs();
        let missing: Vec<String> = repos
            .iter()
            .filter(|r| !entries.get(*r).is_some_and(fresh))
            .cloned()
            .collect();
        if !missing.is_empty() {
            for (repo, tag) in github.latest(&missing) {
                if !tag.is_empty() {
                    entries.insert(repo, Entry { tag, checked: at });
                }
            }
            self.write(&entries);
        }
        repos
            .iter()
            .map(|r| {
                (
                    r.clone(),
                    entries.get(r).map(|e| e.tag.clone()).unwrap_or_default(),
                )
            })
            .collect()
    }

    fn read(&self) -> HashMap<String, Entry> {
        std::fs::read(&self.path)
            .ok()
            .and_then(|raw| serde_json::from_slice(&raw).ok())
            .unwrap_or_default()
    }

    fn write(&self, entries: &HashMap<String, Entry>) {
        let Some(dir) = self.path.parent() else {
            return;
        };
        let Ok(body) = serde_json::to_vec_pretty(entries) else {
            return;
        };
        if std::fs::create_dir_all(dir).is_err() {
            return;
        }
        let staged = dir.join(format!(".latest.json.new.{}", std::process::id()));
        if std::fs::write(&staged, body).is_ok() && std::fs::rename(&staged, &self.path).is_err() {
            let _ = std::fs::remove_file(&staged);
        }
    }
}
