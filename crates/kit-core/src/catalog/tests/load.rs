use std::sync::{Arc, Mutex};

use super::super::{FetchError, Manifest, EMBEDDED, REMOTE_URL};
use super::{loader, write};

const REMOTE: &str = "version: 1\ntools:\n  - name: remote\n    bin: remote\n";

#[test]
fn the_embedded_catalog_lists_every_suite_tool() {
    let m = Manifest::parse(EMBEDDED).unwrap();
    assert!(m.get("sablier").is_some());
    assert!(m.get("SABLIER").is_some(), "lookups ignore case");
    assert!(m
        .tools
        .iter()
        .all(|t| !t.repo.is_empty() && !t.bin.is_empty()));
}

#[test]
fn a_catalog_with_an_unknown_key_still_parses() {
    let raw =
        "version: 1\nfuture: true\ntools:\n  - name: a\n    bin: a\n    shinyNewField: [1, 2]\n";
    assert_eq!(Manifest::parse(raw).unwrap().names(), vec!["a"]);
}

#[test]
fn a_catalog_with_no_tools_is_an_error() {
    assert!(Manifest::parse("version: 1\n").is_err());
}

#[test]
fn the_auth_block_survives_a_write() {
    let m = Manifest::parse(EMBEDDED).unwrap();
    let again = Manifest::parse(&serde_yaml::to_string(&m).unwrap()).unwrap();
    assert_eq!(m.tools, again.tools);
    assert!(again.get("opus").and_then(|t| t.auth.as_ref()).is_some());
}

#[test]
fn a_fetched_catalog_is_cached_and_used() {
    let dir = tempfile::tempdir().unwrap();
    let urls = Arc::new(Mutex::new(Vec::new()));
    let seen = urls.clone();
    let mut l = loader(dir.path());
    l.fetch = Box::new(move |url: &str| {
        seen.lock().unwrap().push(url.to_string());
        Ok::<_, FetchError>(REMOTE.as_bytes().to_vec())
    });
    assert_eq!(l.load_base().unwrap().names(), vec!["remote"]);
    assert_eq!(std::fs::read_to_string(l.cache_path()).unwrap(), REMOTE);
    assert_eq!(*urls.lock().unwrap(), vec![REMOTE_URL.to_string()]);
}

#[test]
fn a_fresh_cache_skips_the_network() {
    let dir = tempfile::tempdir().unwrap();
    let mut l = loader(dir.path());
    write(&l.cache_dir, "tools.yml", REMOTE);
    l.fetch = Box::new(|url: &str| -> Result<Vec<u8>, FetchError> { panic!("fetched {url}") });
    assert_eq!(l.load_base().unwrap().names(), vec!["remote"]);
}

#[test]
fn a_stale_cache_beats_the_embedded_copy_when_offline() {
    let dir = tempfile::tempdir().unwrap();
    let l = loader(dir.path());
    let cache = write(&l.cache_dir, "tools.yml", REMOTE);
    let old = std::time::SystemTime::now() - std::time::Duration::from_secs(48 * 3600);
    std::fs::File::options()
        .write(true)
        .open(cache)
        .unwrap()
        .set_modified(old)
        .unwrap();
    assert_eq!(l.load_base().unwrap().names(), vec!["remote"]);
}

#[test]
fn refresh_fails_loudly_when_offline_and_load_does_not() {
    let dir = tempfile::tempdir().unwrap();
    let l = loader(dir.path());
    assert!(l.refresh().is_err());
    assert!(l.load().unwrap().get("opus").is_some());
}
