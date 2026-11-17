use super::super::parse_sources;

fn names(raw: &str) -> Vec<String> {
    parse_sources(raw)
        .unwrap()
        .single
        .into_iter()
        .map(|s| s.name)
        .collect()
}

#[test]
fn legacy_format() {
    let raw = "single:\n  - name: mytool\n    repo: me/mytool\nlists:\n  - name: community\n    url: https://example.com/list\n";
    let cfg = parse_sources(raw).unwrap();
    assert_eq!(cfg.single.len(), 1);
    assert_eq!(cfg.single[0].name, "mytool");
    assert_eq!(cfg.lists.len(), 1);
    assert_eq!(cfg.lists[0].name, "community");
}

#[test]
fn sources_section_map_and_slice() {
    assert_eq!(
        names("sources:\n  palette: saravenpi/palette\n  pres: saravenpi/pres\n"),
        ["palette", "pres"]
    );
    assert_eq!(
        names("sources:\n  - saravenpi/palette\n  - saravenpi/pres\n"),
        ["palette", "pres"]
    );
}

#[test]
fn sources_section_structured() {
    let raw = "sources:\n  tools:\n    palette: saravenpi/palette\n  lists:\n    - https://example.com/repos.txt\n";
    let cfg = parse_sources(raw).unwrap();
    assert_eq!(cfg.single.len(), 1);
    assert_eq!(cfg.single[0].name, "palette");
    assert_eq!(cfg.lists.len(), 1);
    assert_eq!(cfg.lists[0].name, "repos");
}

#[test]
fn case_insensitive_keys() {
    let cfg = parse_sources(
        "Tools:\n  palette: saravenpi/palette\nLists:\n  - https://example.com/repos.txt\n",
    )
    .unwrap();
    assert_eq!(cfg.single.len(), 1);
    assert_eq!(cfg.single[0].name, "palette");
    assert_eq!(cfg.lists[0].name, "repos");
}

#[test]
fn empty_and_broken_files() {
    assert_eq!(parse_sources("").unwrap(), Default::default());
    assert_eq!(parse_sources("  \n").unwrap(), Default::default());
    assert!(parse_sources("invalid: yaml: [").is_err());
    assert!(parse_sources("sources: 3\n").is_err());
}
