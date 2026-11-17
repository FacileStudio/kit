use std::fs;

use super::attr::env_expr;
use super::stub::{nix_call, path_str, strings, Stub, META};
use super::State;

#[test]
fn sync_runs_build_then_prune_and_records() {
    let stub = Stub::new();
    stub.set("nix.out", META);
    let state = stub
        .nix()
        .sync(&strings(&["ripgrep", "hello", "ripgrep"]))
        .unwrap();
    let profile = path_str(&stub.nix().profile());
    let expr = env_expr("abc123", &strings(&["hello", "ripgrep"])).unwrap();
    let meta = nix_call(&["flake", "metadata", "nixpkgs", "--json"]);
    let build = nix_call(&[
        "build",
        "--impure",
        "--no-link",
        "--profile",
        &profile,
        "--expr",
        &expr,
    ]);
    let prune = ["nix-env", "-p", &profile, "--delete-generations", "+10"];
    assert_eq!(stub.calls(), vec![meta, build, strings(&prune)]);
    let want = State {
        rev: "abc123".into(),
        attrs: strings(&["hello", "ripgrep"]),
        stale: false,
    };
    assert_eq!(state, want);
    assert_eq!(State::load(&stub.state()).unwrap(), Some(want));
    assert!(profile.ends_with("/state/nix-profile"));
}

#[test]
fn sync_rejects_bad_attr_before_running_anything() {
    let stub = Stub::new();
    assert!(stub.nix().sync(&strings(&["hello", "rm -rf"])).is_err());
    assert!(stub.calls().is_empty());
}

#[test]
fn needs_resync_tracks_rev_and_attrs() {
    let stub = Stub::new();
    stub.set("nix.out", META);
    let nix = stub.nix();
    assert!(nix.needs_resync(&strings(&["hello"])).unwrap());
    nix.sync(&strings(&["hello"])).unwrap();
    assert!(!nix.needs_resync(&strings(&["hello", "hello"])).unwrap());
    assert!(nix.needs_resync(&strings(&["hello", "jq"])).unwrap());
    stub.set("nix.out", r#"{"locked":{"rev":"def456"}}"#);
    assert!(nix.needs_resync(&strings(&["hello"])).unwrap());
    State::stale().save(&stub.state()).unwrap();
    stub.set("nix.out", META);
    assert!(nix.needs_resync(&strings(&["hello"])).unwrap());
}

#[test]
fn undo_rolls_back_and_reads_the_generation_record() {
    let stub = Stub::new();
    let profile = stub.nix().profile();
    fs::create_dir_all(profile.join("share/kit")).unwrap();
    fs::write(
        profile.join("share/kit/env.json"),
        r#"{"rev":"old1","attrs":["jq"]}"#,
    )
    .unwrap();
    let state = stub.nix().undo().unwrap();
    assert_eq!(
        stub.calls(),
        vec![strings(&[
            "nix-env",
            "-p",
            &path_str(&profile),
            "--rollback"
        ])]
    );
    let want = State {
        rev: "old1".into(),
        attrs: strings(&["jq"]),
        stale: false,
    };
    assert_eq!(state, want);
    assert_eq!(State::load(&stub.state()).unwrap(), Some(want));
}

#[test]
fn undo_without_a_record_marks_state_stale() {
    let stub = Stub::new();
    assert!(stub.nix().undo().unwrap().stale);
    assert!(State::load(&stub.state()).unwrap().unwrap().stale);
}

#[test]
fn failed_rollback_keeps_state() {
    let stub = Stub::new();
    stub.set("nix-env.code", "1");
    stub.set(
        "nix-env.err",
        "error: no generation older than the current (1) exists\n",
    );
    assert!(stub.nix().undo().is_err());
    assert_eq!(State::load(&stub.state()).unwrap(), None);
}
