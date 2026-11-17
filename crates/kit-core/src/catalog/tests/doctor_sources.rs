use super::super::{SingleSource, SourceConfig};
use super::{loader, write};

fn shadowed(catalog_name: &str, source_name: &str) -> Vec<String> {
    let dir = tempfile::tempdir().unwrap();
    let mut l = loader(dir.path());
    let body = format!("version: 1\ntools:\n  - name: {catalog_name}\n    bin: sablier\n    repo: FacileStudio/sablier\n");
    l.local = Some(write(dir.path(), "tools.yml", &body));
    let cfg = SourceConfig {
        single: vec![SingleSource {
            name: source_name.into(),
            repo: "someone/tool".into(),
            ..Default::default()
        }],
        lists: Vec::new(),
    };
    l.load_base().unwrap().shadowed_by(&cfg)
}

#[test]
fn check_source_shadowing_detects_collision() {
    assert_eq!(shadowed("sablier", "sablier"), ["sablier"]);
}

#[test]
fn check_source_shadowing_no_false_positive() {
    assert!(shadowed("sablier", "my-unique-tool").is_empty());
}

#[test]
fn check_source_shadowing_case_insensitive() {
    assert_eq!(shadowed("Sablier", "sablier"), ["sablier"]);
}
