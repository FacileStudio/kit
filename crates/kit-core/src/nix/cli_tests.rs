use super::stub::{nix_call, strings, Stub, META};
use super::{Nix, NixError};

#[test]
fn registry_rev_runs_flake_metadata() {
    let stub = Stub::new();
    stub.set("nix.out", META);
    assert_eq!(stub.nix().registry_rev().unwrap(), "abc123");
    assert_eq!(
        stub.calls(),
        vec![nix_call(&["flake", "metadata", "nixpkgs", "--json"])]
    );
}

#[test]
fn exists_runs_nix_eval_and_reads_bool() {
    let stub = Stub::new();
    stub.set("nix.out", "true\n");
    assert!(stub.nix().exists("python3Packages.requests").unwrap());
    stub.set("nix.out", "false\n");
    assert!(!stub.nix().exists("nope").unwrap());
    let expr = super::attr::exists_expr("python3Packages.requests").unwrap();
    assert_eq!(
        stub.calls()[0],
        nix_call(&["eval", "--impure", "--expr", &expr])
    );
    stub.set("nix.out", "maybe\n");
    assert!(matches!(
        stub.nix().exists("hello"),
        Err(NixError::BadOutput(_))
    ));
}

#[test]
fn exists_rejects_injection_without_running_nix() {
    let stub = Stub::new();
    for bad in ["a\"b", "$x", "a;b", "a b"] {
        assert!(
            matches!(stub.nix().exists(bad), Err(NixError::InvalidAttr(_))),
            "{bad:?}"
        );
    }
    assert!(stub.calls().is_empty());
}

#[test]
fn missing_binary_is_not_available() {
    let dir = tempfile::tempdir().unwrap();
    let nix = Nix::with_binaries(
        dir.path().join("no-nix"),
        dir.path().join("no-nix-env"),
        dir.path(),
    );
    assert!(matches!(nix.registry_rev(), Err(NixError::NotAvailable(_))));
    assert!(matches!(nix.undo(), Err(NixError::NotAvailable(_))));
}

#[test]
fn failure_carries_the_stderr_tail() {
    let stub = Stub::new();
    stub.set("nix.code", "1");
    let lines: Vec<String> = (1..=30).map(|i| format!("line {i}")).collect();
    stub.set("nix.err", &lines.join("\n"));
    match stub.nix().exists("hello") {
        Err(NixError::CommandFailed { command, stderr }) => {
            assert_eq!(command, "nix eval");
            assert_eq!(stderr, lines[10..].join("\n"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
#[ignore = "needs a real nix; run in CI's nix job with --ignored"]
fn real_nix_resolves_and_syncs() {
    let dir = tempfile::tempdir().unwrap();
    let nix = Nix::with_binaries("nix", "nix-env", dir.path());
    assert!(!nix.registry_rev().unwrap().is_empty());
    assert!(nix.exists("hello").unwrap());
    assert!(!nix.exists("kitNoSuchPackage").unwrap());
    let state = nix.sync(&strings(&["hello"])).unwrap();
    assert!(nix.profile().join("bin/hello").exists());
    assert!(!nix.needs_resync(&state.attrs).unwrap());
    assert!(nix.profile().join("share/kit/env.json").exists());
}
