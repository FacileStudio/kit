use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::super::{
    archive_name, Backend, FacileError, InstallOptions, Owner, Quiet, UpdateOptions, UpdateOutcome,
};
use super::{backend, stub_binary, tar_gz};
use crate::catalog::Tool;

const SCRIPT: &str =
    "#!/bin/sh\nif [ \"$1\" = install ]; then touch \"$0.mcp\"; exit 0; fi\necho 'opus 1.0.0'\n";

fn opus() -> Tool {
    Tool {
        name: "opus".into(),
        bin: "opus".into(),
        asset: "opus".into(),
        repo: "o/opus".into(),
        branch: "main".into(),
        build: "unknown".into(),
        ..Tool::default()
    }
}

fn release_backend(home: &Path, tamper: bool) -> Backend {
    let archive = tar_gz(&[("opus", SCRIPT)]);
    let name = archive_name("opus", "1.0.0");
    let sum = hex::encode(Sha256::digest(&archive));
    let sums = format!("{sum}  {name}\n");
    let served = if tamper {
        tar_gz(&[("opus", "#!/bin/sh\necho evil\n")])
    } else {
        archive
    };
    let (b, _) = backend(home, move |_, path, _| match path {
        "/o/opus/releases/latest" => (200, b"/o/opus/releases/tag/v1.0.0".to_vec()),
        p if p.ends_with("/checksums.txt") => (200, sums.clone().into_bytes()),
        p if p.ends_with(&name) => (200, served.clone()),
        _ => (404, Vec::new()),
    });
    b
}

fn opts(bin_dir: &Path) -> InstallOptions {
    InstallOptions {
        bin_dir: bin_dir.to_path_buf(),
        mcp_tools: vec!["opus".into()],
        ..InstallOptions::default()
    }
}

#[test]
fn install_places_verifies_and_registers_mcp() {
    let home = tempfile::tempdir().unwrap();
    let bin = home.path().join("bin");
    let line = release_backend(home.path(), false)
        .install(&opus(), &opts(&bin), &Quiet)
        .unwrap();
    assert_eq!(line, "opus 1.0.0");
    assert!(
        bin.join("opus.mcp").exists(),
        "an mcp tool must run `<bin> install`"
    );
}

#[test]
fn install_aborts_on_a_checksum_mismatch() {
    let home = tempfile::tempdir().unwrap();
    let bin = home.path().join("bin");
    let err = release_backend(home.path(), true)
        .install(&opus(), &opts(&bin), &Quiet)
        .unwrap_err();
    assert!(
        err.is_integrity(),
        "a tampered archive must abort, got {err}"
    );
    assert!(
        !bin.join("opus").exists(),
        "nothing may be placed after a mismatch"
    );
}

#[test]
fn update_refuses_a_binary_another_manager_owns() {
    let home = tempfile::tempdir().unwrap();
    let bin = home.path().join("bin");
    let mise = home.path().join("mise/installs/opus/1.0.0");
    stub_binary(&mise, "opus", "opus 0.1.0");
    fs::create_dir_all(&bin).unwrap();
    std::os::unix::fs::symlink(mise.join("opus"), bin.join("opus")).unwrap();
    let options = UpdateOptions {
        bin_dir: bin.clone(),
        force: true,
        ..UpdateOptions::default()
    };
    let results = release_backend(home.path(), false).update(&[opus()], &options, &Quiet);
    assert!(matches!(
        results[0].1,
        UpdateOutcome::Refused {
            owner: Owner::Mise,
            ..
        }
    ));
    let err = release_backend(home.path(), false)
        .install(&opus(), &opts(&bin), &Quiet)
        .unwrap_err();
    assert!(matches!(
        err,
        FacileError::Foreign {
            owner: Owner::Mise,
            ..
        }
    ));
}

#[test]
fn update_skips_a_current_tool_and_installs_a_missing_one() {
    let home = tempfile::tempdir().unwrap();
    let bin = home.path().join("bin");
    let options = UpdateOptions {
        bin_dir: bin.clone(),
        ..UpdateOptions::default()
    };
    let b = release_backend(home.path(), false);
    let first = b.update(&[opus()], &options, &Quiet);
    assert!(matches!(&first[0].1, UpdateOutcome::Updated(line) if line == "opus 1.0.0"));
    assert!(
        b.update(&[opus()], &options, &Quiet).is_empty(),
        "a current tool must not download again"
    );
}
