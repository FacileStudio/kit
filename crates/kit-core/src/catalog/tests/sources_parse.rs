use super::super::parse_sources;

fn names_and_repos(raw: &str) -> Vec<(String, String, String)> {
    parse_sources(raw)
        .unwrap()
        .single
        .into_iter()
        .map(|s| (s.name, s.repo, s.branch))
        .collect()
}

fn triple(name: &str, repo: &str, branch: &str) -> (String, String, String) {
    (name.into(), repo.into(), branch.into())
}

#[test]
fn root_level_map() {
    let got = names_and_repos("palette: saravenpi/palette\npres: saravenpi/pres\n");
    assert_eq!(
        got,
        vec![
            triple("palette", "saravenpi/palette", ""),
            triple("pres", "saravenpi/pres", "")
        ]
    );
}

#[test]
fn branch_pin() {
    let got =
        names_and_repos("palette: saravenpi/palette@v1.2.0\nripgrep: BurntSushi/ripgrep@beta\n");
    assert_eq!(got[0].2, "v1.2.0");
    assert_eq!(got[1].2, "beta");
}

#[test]
fn bare_sequence() {
    let got = names_and_repos("- saravenpi/palette\n- saravenpi/pres@main\n");
    assert_eq!(
        got,
        vec![
            triple("palette", "saravenpi/palette", ""),
            triple("pres", "saravenpi/pres", "main")
        ]
    );
}

#[test]
fn tools_section() {
    let cfg = parse_sources(
        "tools:\n  palette: saravenpi/palette\nlists:\n  - https://example.com/repos.txt\n",
    )
    .unwrap();
    assert_eq!(cfg.single.len(), 1);
    assert_eq!(cfg.single[0].name, "palette");
    assert_eq!(cfg.lists.len(), 1);
    assert_eq!(cfg.lists[0].url, "https://example.com/repos.txt");
    assert_eq!(cfg.lists[0].name, "repos");
}

#[test]
fn extended_object() {
    let cfg =
        parse_sources("custom:\n  repo: myorg/custom\n  build: rust\n  bin: custom-bin\n").unwrap();
    assert_eq!(cfg.single.len(), 1);
    let s = &cfg.single[0];
    assert_eq!(
        (s.name.as_str(), s.repo.as_str()),
        ("custom", "myorg/custom")
    );
    assert_eq!((s.build.as_str(), s.bin.as_str()), ("rust", "custom-bin"));
}
