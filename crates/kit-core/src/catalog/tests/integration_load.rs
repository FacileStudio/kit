use super::super::{sources_path, sources_paths_in};
use super::{loader, with_sources, write};

const MYTOOL: &str = "single:\n  - name: mytool\n    repo: me/mytool\n    bin: mytool\n";

#[test]
fn sources_file_loading() {
    let home = tempfile::tempdir().unwrap();
    let l = with_sources(
        home.path(),
        "single:\n  - name: mytool\n    repo: me/mytool\n    bin: mytool\n    build: go\n\
         lists:\n  - name: community\n    url: https://example.com/list\n",
    );
    let cfg = l.load_sources().unwrap();
    assert_eq!(cfg.single.len(), 1);
    assert_eq!(
        (cfg.single[0].name.as_str(), cfg.single[0].repo.as_str()),
        ("mytool", "me/mytool")
    );
    assert_eq!(cfg.lists.len(), 1);
    assert_eq!(cfg.lists[0].name, "community");

    let empty = loader(home.path());
    assert_eq!(empty.load_sources().unwrap(), Default::default());

    let mut missing = loader(home.path());
    missing.sources = sources_paths_in(
        "/nonexistent/deadbeef12345".as_ref(),
        "/nonexistent/deadbeef12345/.config/kit".as_ref(),
    );
    assert_eq!(missing.load_sources().unwrap(), Default::default());
}

#[test]
fn sources_path_priority() {
    let home = tempfile::tempdir().unwrap();
    let h = home.path();
    let candidates = sources_paths_in(h, &h.join(".config/kit"));
    let legacy = write(h, ".config/facile/sources.yml", "single: []\n");
    assert_eq!(sources_path(&candidates), Some(legacy));
    let home_file = write(h, ".facile.yml", "single: []\n");
    assert_eq!(
        sources_path(&candidates),
        Some(home_file),
        "the home file has priority"
    );
    let kit = write(h, ".config/kit/config.yml", "single: []\n");
    assert_eq!(
        sources_path(&candidates),
        Some(kit),
        "kit's own config wins over the legacy files"
    );
}

#[test]
fn config_yml_with_sources_key() {
    let home = tempfile::tempdir().unwrap();
    let h = home.path();
    write(
        h,
        ".config/facile/config.yml",
        "sources:\n  palette: saravenpi/palette\n  pres: saravenpi/pres\n",
    );
    let mut l = loader(h);
    l.sources = sources_paths_in(h, &h.join(".config/kit"));
    let names: Vec<_> = l
        .load_sources()
        .unwrap()
        .single
        .into_iter()
        .map(|s| s.name)
        .collect();
    assert_eq!(names, ["palette", "pres"]);
}

#[test]
fn single_sources_shadow_base_and_append_new() {
    let home = tempfile::tempdir().unwrap();
    let l = with_sources(
        home.path(),
        "single:\n  - name: sablier\n    repo: me/my-sablier-fork\n    bin: sablier\n    build: rust\n\
         \x20   summary: My custom sablier\n  - name: mytool\n    repo: me/mytool\n    bin: mytool\n    build: go\n",
    );
    let m = l.load().unwrap();
    let sablier = m.get("sablier").unwrap();
    assert_eq!(sablier.repo, "me/my-sablier-fork");
    assert_eq!(sablier.summary, "My custom sablier");
    assert_eq!(m.get("mytool").unwrap().repo, "me/mytool");
    assert_eq!(m.get("opus").unwrap().repo, "FacileStudio/opus-cli");
}

#[test]
fn load_merges_user_sources_and_load_base_does_not() {
    let home = tempfile::tempdir().unwrap();
    let l = with_sources(home.path(), MYTOOL);
    assert!(l.load_base().unwrap().get("mytool").is_none());
    let m = l.load().unwrap();
    assert!(m.get("mytool").is_some());
    assert!(m.get("opus").is_some());
}

#[test]
fn load_keeps_a_parse_error_in_load_errors() {
    let home = tempfile::tempdir().unwrap();
    let l = with_sources(home.path(), "invalid: yaml: [");
    let m = l.load().unwrap();
    assert!(!m.load_errors.is_empty());
    assert!(m.get("opus").is_some());
}
