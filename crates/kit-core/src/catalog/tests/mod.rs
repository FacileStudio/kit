use std::fs;
use std::path::{Path, PathBuf};

use super::{FetchError, Loader};

mod discover;
mod doctor_sources;
mod integration_load;
mod integration_merge;
mod integration_parse;
mod load;
mod manifest;
mod sources_parse;
mod sources_parse_more;

fn offline(url: &str) -> Result<Vec<u8>, FetchError> {
    Err(FetchError::Transport {
        url: url.to_string(),
        message: "offline".into(),
    })
}

fn loader(dir: &Path) -> Loader {
    Loader {
        cache_dir: dir.join("cache"),
        sources: Vec::new(),
        local: None,
        fetch: Box::new(offline),
    }
}

fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, body).unwrap();
    path
}

fn with_sources(dir: &Path, body: &str) -> Loader {
    let mut l = loader(dir);
    l.sources = vec![write(dir, ".facile.yml", body)];
    l
}
