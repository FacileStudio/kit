use std::path::{Path, PathBuf};

use super::super::install::Job;
use super::super::{FacileError, InstallOptions, Quiet};
use super::{backend, offline};
use crate::catalog::Tool;

fn fake_tool() -> Tool {
    Tool {
        bin: "tool".into(),
        asset: "tool".into(),
        repo: "FacileStudio/tool".into(),
        branch: "main".into(),
        build: "unknown".into(),
        ..Tool::default()
    }
}

fn build_with(release_error: FacileError) -> Result<Vec<PathBuf>, FacileError> {
    let home = tempfile::tempdir().unwrap();
    let (b, _) = backend(home.path(), offline);
    let message = release_error.to_string();
    let integrity = release_error.is_integrity();
    let release = move |_: &Tool, _: &str, _: &Path| -> Result<Vec<PathBuf>, FacileError> {
        match integrity {
            true => Err(FacileError::Integrity(message.clone())),
            false => Err(FacileError::Download(message.clone())),
        }
    };
    let opts = InstallOptions::default();
    let job = Job {
        tool: &fake_tool(),
        opts: &opts,
        work: home.path(),
    };
    b.build(&job, &release, &Quiet)
}

#[test]
fn build_refuses_a_source_fallback_after_an_integrity_failure() {
    let err = build_with(FacileError::Integrity(
        "checksum mismatch for tool.tar.gz".into(),
    ))
    .unwrap_err();
    assert!(
        err.is_integrity(),
        "an integrity failure must abort the install, got {err}"
    );
}

#[test]
fn build_still_falls_back_when_a_release_is_merely_unreachable() {
    let err = build_with(FacileError::Download(
        "tool.tar.gz returned status 404".into(),
    ))
    .unwrap_err();
    assert!(
        err.to_string().contains("unknown build backend"),
        "must reach the source build, got {err}"
    );
}
