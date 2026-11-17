use super::super::{stale, up_to_date, FacileError, Quiet};
use super::stub_binary;
use crate::catalog::Tool;

fn tag(_: &str) -> Result<String, FacileError> {
    Ok("v1.2.3".to_string())
}

#[test]
fn up_to_date_with_version_cmd() {
    let dir = tempfile::tempdir().unwrap();
    let opus = stub_binary(dir.path(), "opus", "opus v1.2.3 (abc123)");
    let mut t = Tool {
        name: "opus".into(),
        bin: "opus".into(),
        repo: "FacileStudio/opus".into(),
        ..Tool::default()
    };
    t.version_cmd = "version".into();
    t.version_pattern = r"v(\d+\.\d+\.\d+)".into();
    assert!(
        up_to_date("opus 1.2.3", &opus, &t, &tag),
        "versionCmd output must match"
    );
    let missing = dir.path().join("missing");
    assert!(
        up_to_date("opus 1.2.3", &missing, &t, &tag),
        "a failed versionCmd falls through"
    );
}

fn named(name: &str, repo: &str) -> Tool {
    Tool {
        name: name.into(),
        bin: name.into(),
        asset: name.into(),
        repo: repo.into(),
        ..Tool::default()
    }
}

#[test]
fn stale_keeps_an_installed_tool_whose_latest_tag_is_unreadable() {
    let dir = tempfile::tempdir().unwrap();
    stub_binary(dir.path(), "opus", "opus 0.9.0");
    let tools = [
        named("opus", "FacileStudio/opus-cli"),
        named("nuage", "FacileStudio/nuage-cli"),
    ];
    let tag_of = |_: &str| Err(FacileError::Download("context deadline exceeded".into()));
    let got: Vec<_> = stale(&tools, dir.path(), &tag_of, &Quiet)
        .into_iter()
        .map(|t| t.name)
        .collect();
    assert_eq!(got, ["nuage"]);
}
