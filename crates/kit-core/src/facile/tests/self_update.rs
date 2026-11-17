use std::fs;

use super::super::{Owner, Quiet, SelfOptions, SelfOutcome};
use super::{backend, stub_binary};
use crate::catalog::Tool;

#[test]
fn self_update_leaves_a_current_kit_and_a_source_build_alone() {
    let home = tempfile::tempdir().unwrap();
    let (b, _) = backend(home.path(), |_, path, _| match path {
        "/FacileStudio/kit/releases/latest" => {
            (200, b"/FacileStudio/kit/releases/tag/v0.1.0".to_vec())
        }
        _ => (404, Vec::new()),
    });
    let exe = stub_binary(&home.path().join("bin"), "kit", "kit 0.1.0");
    let current = b
        .self_update("0.1.0", &exe, SelfOptions::default(), &Quiet)
        .unwrap();
    assert_eq!(current, SelfOutcome::UpToDate("0.1.0".into()));
    let ahead = b
        .self_update("0.2.0", &exe, SelfOptions::default(), &Quiet)
        .unwrap();
    assert_eq!(ahead, SelfOutcome::UpToDate("0.2.0".into()));
    let source = b
        .self_update("edf2b6f", &exe, SelfOptions::default(), &Quiet)
        .unwrap();
    assert_eq!(source, SelfOutcome::SourceBuild("edf2b6f".into()));
}

#[test]
fn self_update_refuses_under_a_package_manager() {
    let home = tempfile::tempdir().unwrap();
    let (b, _) = backend(home.path(), |_, _, _| (404, Vec::new()));
    let exe = stub_binary(
        &home.path().join("mise/installs/kit/0.1.0"),
        "kit",
        "kit 0.1.0",
    );
    let got = b
        .self_update(
            "0.1.0",
            &exe,
            SelfOptions {
                force: true,
                source: false,
            },
            &Quiet,
        )
        .unwrap();
    let want = SelfOutcome::Managed {
        owner: Owner::Mise,
        behind: false,
        command: "mise upgrade".into(),
    };
    assert_eq!(got, want);
}

#[test]
fn installed_runs_the_binaries_and_skips_broken_ones() {
    let home = tempfile::tempdir().unwrap();
    let bin = home.path().join("bin");
    stub_binary(&bin, "opus", "opus version 1.0.0");
    fs::write(bin.join("nuage"), "not executable").unwrap();
    let manifest = crate::catalog::Manifest {
        tools: vec![
            Tool {
                name: "opus".into(),
                bin: "opus".into(),
                ..Tool::default()
            },
            Tool {
                name: "nuage".into(),
                bin: "nuage".into(),
                ..Tool::default()
            },
        ],
        ..Default::default()
    };
    let (b, _) = backend(home.path(), |_, _, _| (404, Vec::new()));
    let found = b.installed(&manifest, &bin);
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].version.as_str(), found[0].owner),
        ("1.0.0", Owner::Kit)
    );
}
