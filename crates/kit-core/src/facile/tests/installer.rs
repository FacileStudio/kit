use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use sha2::{Digest, Sha256};

use super::super::archive::{extract, verify_checksum};
use super::super::place::atomic_install;
use super::super::runlog::{last_lines, run};
use super::super::FacileError;
use super::tar_gz;

#[test]
fn verify_checksum_rejects_a_tampered_archive() {
    let blob = b"the real artifact";
    let sums = format!(
        "{}  tool_1.0.0_darwin_arm64.tar.gz\n",
        hex::encode(Sha256::digest(blob))
    );
    verify_checksum(blob, "tool_1.0.0_darwin_arm64.tar.gz", &sums).unwrap();
    let swapped = verify_checksum(b"swapped", "tool_1.0.0_darwin_arm64.tar.gz", &sums);
    assert!(
        swapped.unwrap_err().is_integrity(),
        "a mismatch must abort as an integrity failure"
    );
    let unlisted = verify_checksum(blob, "absent.tar.gz", &sums);
    assert!(
        unlisted.unwrap_err().is_integrity(),
        "an unlisted archive must abort as an integrity failure"
    );
    let transport =
        FacileError::Download("tool_1.0.0_darwin_arm64.tar.gz returned status 404".into());
    assert!(
        !transport.is_integrity(),
        "a transport failure must stay a source-build fallback"
    );
}

#[test]
fn extract_finds_the_binary_and_ignores_the_rest() {
    let archive = tar_gz(&[
        ("README.md", "not the binary"),
        ("opus", "#!/bin/sh\necho opus 1.2.3\n"),
    ]);
    let work = tempfile::tempdir().unwrap();
    let path = extract(&archive, "opus", work.path()).unwrap();
    assert!(fs::read_to_string(path).unwrap().contains("opus 1.2.3"));
    assert!(extract(&archive, "sablier", work.path()).is_err());
}

#[test]
fn atomic_install_leaves_no_staged_file_behind() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    let dest = dir.path().join("opus");
    fs::write(&src, "new").unwrap();
    fs::write(&dest, "old").unwrap();
    atomic_install(&src, &dest).unwrap();
    assert_eq!(fs::read_to_string(&dest).unwrap(), "new");
    assert_eq!(
        fs::metadata(&dest).unwrap().permissions().mode() & 0o777,
        0o755
    );
    let staged = fs::read_dir(dir.path()).unwrap().filter_map(Result::ok);
    let names: Vec<_> = staged
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert!(
        !names.iter().any(|n| n.starts_with(".opus.new.")),
        "staged file left: {names:?}"
    );
}

#[test]
fn run_keeps_the_compiler_errors_and_the_summary() {
    let script = "echo 'error[E0425]: cannot find value `x`'; echo 'error: could not compile `sidebar`'; exit 1";
    let err = run(Command::new("sh").args(["-c", script]))
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("E0425"),
        "the compiler error must survive, got {err}"
    );
    assert!(
        err.contains("could not compile `sidebar`"),
        "the summary must stay too, got {err}"
    );
}

#[test]
fn last_lines_keeps_the_tail_of_a_build_log() {
    assert_eq!(last_lines("one\ntwo\nthree\nfour\n", 2), "three\nfour");
    assert_eq!(last_lines("", 5), "");
}
