use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::super::lists::cache_path;
use super::super::{Fetch, FetchError, ListSource, SourceConfig};
use super::{loader, with_sources};

fn site(files: &[(&str, &str)]) -> (Box<dyn Fetch>, Arc<Mutex<Vec<String>>>) {
    let map: HashMap<String, String> = files
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let log = Arc::new(Mutex::new(Vec::new()));
    let seen = log.clone();
    let fetch = move |url: &str| {
        seen.lock().unwrap().push(url.to_string());
        map.get(url)
            .map(|b| b.as_bytes().to_vec())
            .ok_or(FetchError::Status {
                url: url.into(),
                code: 404,
            })
    };
    (Box::new(fetch), log)
}

const RAW: &str = "https://raw.githubusercontent.com";

#[test]
fn an_incomplete_single_source_reads_the_repo_facile_toml() {
    let home = tempfile::tempdir().unwrap();
    let mut l = with_sources(home.path(), "- me/widgets@dev\n");
    let toml =
        "name = \"widgets\"\nrepo = \"me/widgets\"\nbuild = \"rust\"\nbins = [\"a\", \"b\"]\n";
    let (fetch, log) = site(&[(&format!("{RAW}/me/widgets/dev/facile.toml"), toml)]);
    l.fetch = fetch;
    let m = l.load().unwrap();
    let w = m.get("widgets").unwrap();
    assert_eq!((w.build.as_str(), w.branch.as_str()), ("rust", "dev"));
    assert_eq!(w.bin_list(), ["a", "b"]);
    assert!(log
        .lock()
        .unwrap()
        .iter()
        .any(|u| u.ends_with("/me/widgets/dev/facile.toml")));
}

#[test]
fn mise_toml_is_the_fallback_on_master() {
    let home = tempfile::tempdir().unwrap();
    let mut l = with_sources(home.path(), "- me/tool\n");
    let mise = "[tools]\nnode = \"20\"\n[facile]\nname = \"tool\"\nrepo = \"me/tool\"\n";
    (l.fetch, _) = site(&[(&format!("{RAW}/me/tool/master/mise.toml"), mise)]);
    let m = l.load().unwrap();
    assert_eq!(m.get("tool").unwrap().repo, "me/tool");
    assert!(m.load_errors.is_empty(), "{:?}", m.load_errors);
}

#[test]
fn a_repo_with_no_config_falls_back_to_defaults_or_errors_without_a_name() {
    let home = tempfile::tempdir().unwrap();
    let mut l = with_sources(home.path(), "- me/named\n- repo: me/nameless\n");
    (l.fetch, _) = site(&[]);
    let m = l.load().unwrap();
    let named = m.get("named").unwrap();
    assert_eq!((named.bin.as_str(), named.build.as_str()), ("named", "go"));
    assert!(
        m.load_errors.iter().any(|e| e.contains("me/nameless")),
        "{:?}",
        m.load_errors
    );
}

#[test]
fn list_sources_are_discovered_and_cached() {
    let home = tempfile::tempdir().unwrap();
    let mut l = with_sources(home.path(), "lists:\n  - https://example.com/repos.txt\n");
    let toml = "name = \"listed\"\nrepo = \"me/listed\"\n";
    let (fetch, log) = site(&[
        ("https://example.com/repos.txt", "me/listed\n"),
        (&format!("{RAW}/me/listed/main/facile.toml"), toml),
    ]);
    l.fetch = fetch;
    assert!(l.load().unwrap().get("listed").is_some());
    assert!(cache_path(&l.cache_dir, "https://example.com/repos.txt").exists());
    let lists_fetched = |log: &Arc<Mutex<Vec<String>>>| {
        log.lock()
            .unwrap()
            .iter()
            .filter(|u| u.contains("example.com"))
            .count()
    };
    assert_eq!(lists_fetched(&log), 1);
    l.load().unwrap();
    assert_eq!(
        lists_fetched(&log),
        1,
        "the second load reads the list from the cache"
    );
}

#[test]
fn an_unreachable_list_is_a_load_error() {
    let dir = tempfile::tempdir().unwrap();
    let l = loader(dir.path());
    let cfg = SourceConfig {
        single: Vec::new(),
        lists: vec![ListSource {
            name: "gone".into(),
            url: "https://example.com/gone".into(),
        }],
    };
    let (tools, errors) =
        super::super::discover::discover_from_lists(l.fetch.as_ref(), &l.cache_dir, &cfg);
    assert!(tools.is_empty());
    assert!(errors[0].contains("gone"), "{errors:?}");
}

#[test]
#[ignore = "needs the network"]
fn live_fetch_reaches_the_catalog_on_main() {
    let dir = tempfile::tempdir().unwrap();
    let mut l = loader(dir.path());
    l.fetch = Box::new(super::super::HttpFetch::new(
        std::time::Duration::from_secs(10),
    ));
    assert!(l.refresh().unwrap().get("sablier").is_some());
    assert!(l.cache_path().exists());
}
