use super::super::toml_spec::{parse_facile_toml, parse_mise_toml};
use super::super::{match_version, CatalogError, SingleSource, Tool};

fn err_text<T: std::fmt::Debug>(r: Result<T, CatalogError>) -> String {
    r.unwrap_err().to_string()
}

#[test]
fn facile_toml_parsing() {
    let raw = "name = \"mycli\"\nrepo = \"me/mycli\"\nsummary = \"A custom CLI\"\nbranch = \"main\"\nbin = \"mycli\"\n\
               build = \"rust\"\nversionCmd = \"mycli --version\"\nversionPattern = \"v?(\\\\d+\\\\.\\\\d+\\\\.\\\\d+)\"\n";
    let ft = parse_facile_toml(raw.as_bytes()).unwrap();
    assert_eq!((ft.name.as_str(), ft.repo.as_str()), ("mycli", "me/mycli"));
    assert_eq!(ft.summary, "A custom CLI");
    assert_eq!(ft.version_cmd, "mycli --version");
    assert!(err_text(parse_facile_toml(b"repo = \"me/mycli\"\n")).contains("name is required"));
    assert!(err_text(parse_facile_toml(b"name = \"mycli\"\n")).contains("repo is required"));
    let bad = b"name = \"mycli\"\nrepo = \"me/mycli\"\nversionPattern = \"[bad\"\n";
    assert!(err_text(parse_facile_toml(bad)).contains("invalid versionPattern"));
}

#[test]
fn mise_toml_parsing() {
    let raw = "[tools]\nnode = \"20\"\n\n[facile]\nname = \"mycli\"\nrepo = \"me/mycli\"\nsummary = \"A custom CLI\"\nbranch = \"main\"\n";
    let ft = parse_mise_toml(raw.as_bytes()).unwrap();
    assert_eq!((ft.name.as_str(), ft.repo.as_str()), ("mycli", "me/mycli"));
    assert!(matches!(
        parse_mise_toml(b"[tools]\nnode = \"20\"\n"),
        Err(CatalogError::NoFacileToml)
    ));
    assert!(
        err_text(parse_mise_toml(b"[facile]\nrepo = \"me/mycli\"\n")).contains("name is required")
    );
    assert!(err_text(parse_mise_toml(b"[facile]\nname = \"mycli\"\n")).contains("repo is required"));
}

#[test]
fn version_pattern_matching() {
    let cases = [
        ("mycli version 1.2.3", r"(\d+\.\d+\.\d+)", "1.2.3"),
        ("v2.0.1", r"v(\d+\.\d+\.\d+)", "2.0.1"),
        ("1.2.3", r"\d+\.\d+\.\d+", "1.2.3"),
        ("foo", r"(\d+\.\d+\.\d+)", ""),
        ("1.2.3", "[", ""),
        ("", r"(\d+\.\d+\.\d+)", ""),
        ("1.2.3", "", ""),
        (
            "mycli 3.4.5 (abc123) linux/amd64",
            r"mycli (\d+\.\d+\.\d+)",
            "3.4.5",
        ),
        ("1.2.3-4-gabc123", r"(\d+\.\d+\.\d+)-(\d+)", "1.2.3"),
    ];
    for (output, pattern, want) in cases {
        assert_eq!(
            match_version(output, pattern),
            want,
            "{output:?} with {pattern:?}"
        );
    }
}

fn source(name: &str) -> SingleSource {
    SingleSource {
        name: name.into(),
        repo: "me/mycli".into(),
        ..Default::default()
    }
}

fn shape(t: &Tool) -> String {
    format!(
        "{} {} {} {} {}",
        t.branch, t.build, t.bin, t.src_subdir, t.version_cmd
    )
}

#[test]
fn single_source_to_tool_fills_defaults_and_keeps_explicit_values() {
    assert_eq!(
        shape(&source("mycli").to_tool().unwrap()),
        "main go mycli . "
    );
    let explicit = SingleSource {
        branch: "develop".into(),
        build: "rust".into(),
        bin: "mc".into(),
        src_subdir: "src".into(),
        version_cmd: "mc".into(),
        version_pattern: r"v(\d+\.\d+\.\d+)".into(),
        ..source("mycli")
    };
    let t = explicit.to_tool().unwrap();
    assert_eq!(shape(&t), "develop rust mc src mc");
    assert_eq!(t.version_pattern, r"v(\d+\.\d+\.\d+)");
}

#[test]
fn single_source_to_tool_rejects_bad_entries() {
    assert!(err_text(source("").to_tool()).contains("missing name"));
    let bad = SingleSource {
        version_pattern: "[".into(),
        ..source("mycli")
    };
    assert!(err_text(bad.to_tool()).contains("invalid versionPattern"));
}
