use std::fs::File;
use std::time::{Duration, SystemTime};

use super::super::cache::fresh_against;
use super::super::toml_spec::parse_facile_toml;
use super::super::{catalog_override, Manifest, EMBEDDED};
use super::{loader, write};

#[test]
fn cache_is_stale_once_the_binary_is_newer() {
    let dir = tempfile::tempdir().unwrap();
    let cache = write(dir.path(), "tools.yml", "version: 1\n");
    let written = SystemTime::now() - Duration::from_secs(60);
    File::options()
        .write(true)
        .open(&cache)
        .unwrap()
        .set_modified(written)
        .unwrap();
    assert!(!fresh_against(
        &cache,
        Some(written + Duration::from_secs(3600))
    ));
    assert!(fresh_against(
        &cache,
        Some(written - Duration::from_secs(60))
    ));
    assert!(fresh_against(&cache, None));
    let old = SystemTime::now() - Duration::from_secs(25 * 3600);
    File::options()
        .write(true)
        .open(&cache)
        .unwrap()
        .set_modified(old)
        .unwrap();
    assert!(!fresh_against(&cache, None), "a day-old cache is refetched");
}

#[test]
fn kit_catalog_overrides_everything() {
    let dir = tempfile::tempdir().unwrap();
    let mut l = loader(dir.path());
    l.local = Some(write(
        dir.path(),
        "local.yml",
        "version: 1\ntools:\n  - name: only\n    bin: only\n",
    ));
    let m = l.load().unwrap();
    assert_eq!(m.names(), vec!["only"]);
}

#[test]
fn a_missing_override_falls_back_to_embedded() {
    let dir = tempfile::tempdir().unwrap();
    let mut l = loader(dir.path());
    l.local = Some(dir.path().join("absent.yml"));
    assert!(!l.load().unwrap().tools.is_empty());
}

#[test]
fn kit_catalog_wins_over_facile_catalog() {
    let pick =
        |k: Option<&str>, f: Option<&str>| catalog_override(k.map(Into::into), f.map(Into::into));
    assert_eq!(pick(Some("/k"), Some("/f")), Some("/k".into()));
    assert_eq!(pick(None, Some("/f")), Some("/f".into()));
    assert_eq!(pick(Some(""), Some("/f")), Some("/f".into()));
    assert_eq!(pick(None, None), None);
}

#[test]
fn embedded_sso_code_flows_are_on_the_porte_contract() {
    let m = Manifest::parse(EMBEDDED).unwrap();
    for tool in &m.tools {
        let Some(sso) = tool.auth.as_ref().and_then(|a| a.get("sso")) else {
            continue;
        };
        let field = |k: &str| {
            sso.get(k)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        };
        if field("callbackWith") != "code" {
            continue;
        }
        assert_eq!(field("portParam"), "port", "{}", tool.name);
        assert_eq!(field("stateParam"), "cli_state", "{}", tool.name);
        assert!(field("extraParams").contains("flow=cli"), "{}", tool.name);
        assert_eq!(field("callbackPath"), "/", "{}", tool.name);
        assert!(!field("exchangePath").is_empty(), "{}", tool.name);
    }
}

#[test]
fn sonar_the_first_mcp_tool_declares_the_install_hook() {
    let m = Manifest::parse(EMBEDDED).unwrap();
    assert!(m.mcp.iter().any(|n| n == "sonar"), "got {:?}", m.mcp);
}

#[test]
fn one_manifest_declares_every_binary_of_a_repo() {
    let raw = "name = \"widgets\"\nrepo = \"saravenpi/widgets\"\nbuild = \"rust\"\n\
               bins = [\"statusbar\", \"sidebar\", \"launcher\"]\n";
    let tool = parse_facile_toml(raw.as_bytes())
        .unwrap()
        .to_tool()
        .unwrap();
    assert_eq!(tool.bin_list(), vec!["statusbar", "sidebar", "launcher"]);
    assert_eq!(tool.bin, "statusbar");
}

#[test]
fn a_single_bin_still_produces_one_entry() {
    let raw = "name = \"palette\"\nrepo = \"saravenpi/palette\"\nbin = \"palette\"\n";
    let tool = parse_facile_toml(raw.as_bytes())
        .unwrap()
        .to_tool()
        .unwrap();
    assert_eq!(tool.bin_list(), vec!["palette"]);
}
