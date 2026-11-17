use std::path::Path;

use super::super::{stale, up_to_date, FacileError, Quiet};
use super::stub_binary;
use crate::catalog::Tool;

fn tool(name: &str, repo: &str) -> Tool {
    Tool {
        name: name.into(),
        bin: name.into(),
        asset: name.into(),
        repo: repo.into(),
        ..Tool::default()
    }
}

fn tag(t: &'static str) -> impl Fn(&str) -> Result<String, FacileError> + Sync {
    move |_| Ok(t.to_string())
}

type Case = (&'static str, Option<&'static str>, bool);

fn check(cases: &[Case]) {
    let opus = tool("opus", "FacileStudio/opus");
    for &(have, latest, want) in cases {
        let tag_of = move |_: &str| {
            latest
                .map(str::to_string)
                .ok_or_else(|| FacileError::Download("404".into()))
        };
        assert_eq!(
            up_to_date(have, Path::new(""), &opus, &tag_of),
            want,
            "{have} against {latest:?}"
        );
    }
}

#[test]
fn up_to_date_skips_only_an_exact_version_match() {
    check(&[
        ("opus 0.1.0", Some("v0.1.0"), true),
        ("opus 0.1.0", Some("0.1.0"), true),
        ("opus version 0.1.0", Some("v0.1.0"), true),
        ("opus 0.1.0", Some("v0.2.0"), false),
        ("opus 0.3.0", Some("v0.2.0"), false),
    ]);
}

#[test]
fn up_to_date_reinstalls_when_it_cannot_tell() {
    check(&[("opus 0.1.0", None, false), ("", Some("v0.1.0"), false)]);
}

#[test]
fn up_to_date_checks_source_only_tools_against_the_latest_tag() {
    let mut opus = tool("opus", "FacileStudio/opus");
    opus.asset.clear();
    assert!(up_to_date(
        "opus 0.1.0",
        Path::new(""),
        &opus,
        &tag("v0.1.0")
    ));
    assert!(!up_to_date(
        "opus 0.0.9",
        Path::new(""),
        &opus,
        &tag("v0.1.0")
    ));
}

#[test]
fn stale_keeps_the_outdated_tools_in_catalog_order() {
    let dir = tempfile::tempdir().unwrap();
    stub_binary(dir.path(), "opus", "opus 0.1.0");
    stub_binary(dir.path(), "sablier", "sablier 0.1.1");
    stub_binary(dir.path(), "filet", "filet 0.6.1");
    let tools = [
        tool("opus", "FacileStudio/opus"),
        tool("sablier", "FacileStudio/Sablier"),
        tool("nuage", "FacileStudio/Nuage"),
        tool("filet", "FacileStudio/filet"),
    ];
    let tag_of = |repo: &str| {
        Ok(match repo {
            "FacileStudio/opus" => "v0.1.0",
            "FacileStudio/Sablier" => "v0.2.0",
            "FacileStudio/Nuage" => "v0.3.0",
            _ => "v0.6.1",
        }
        .to_string())
    };
    let got: Vec<_> = stale(&tools, dir.path(), &tag_of, &Quiet)
        .into_iter()
        .map(|t| t.name)
        .collect();
    assert_eq!(got, ["sablier", "nuage"]);
}

#[test]
fn up_to_date_with_version_pattern() {
    let dir = tempfile::tempdir().unwrap();
    let opus = stub_binary(dir.path(), "opus", "opus v1.2.3 (abc123)");
    let old = stub_binary(dir.path(), "opus-old", "opus v1.1.0 (def456)");
    let baz = stub_binary(dir.path(), "baz", "baz tool 2.0.0 linux/amd64");
    let cases = [
        (&opus, "opus v1.2.3 (abc123)", r"v(\d+\.\d+\.\d+)", true),
        (&old, "opus v1.1.0 (def456)", r"v(\d+\.\d+\.\d+)", false),
        (&baz, "baz tool 2.0.0 linux/amd64", r"\d+\.\d+\.\d+", true),
        (
            &opus,
            "opus v1.2.3 (abc123)",
            r"release-(\d+\.\d+\.\d+)",
            false,
        ),
    ];
    for (path, have, pattern, want) in cases {
        let mut t = tool("opus", "FacileStudio/opus");
        t.version_pattern = pattern.into();
        let latest = if path == &baz { "v2.0.0" } else { "v1.2.3" };
        assert_eq!(
            up_to_date(have, path, &t, &tag(latest)),
            want,
            "{have} with {pattern}"
        );
    }
}
