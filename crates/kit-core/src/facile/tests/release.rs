use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::super::runlog::ignore_repo_mise;
use super::github;

#[test]
fn release_asset_falls_back_to_the_api_on_server_errors() {
    let (gh, calls) = github(|_, path, headers| match path {
        "/o/tool/releases/download/v1.0.0/tool.tar.gz" => (504, Vec::new()),
        "/repos/o/tool/releases/tags/v1.0.0" => (
            200,
            br#"{"assets":[{"id":7,"name":"tool.tar.gz"}]}"#.to_vec(),
        ),
        "/repos/o/tool/releases/assets/7"
            if headers.contains(&("Accept", "application/octet-stream".into())) =>
        {
            (200, b"bytes".to_vec())
        }
        "/repos/o/tool/releases/assets/7" => (406, Vec::new()),
        _ => (404, Vec::new()),
    });
    assert_eq!(
        gh.release_asset("o/tool", "v1.0.0", "tool.tar.gz").unwrap(),
        b"bytes"
    );
    let downloads = calls
        .lock()
        .unwrap()
        .iter()
        .filter(|c| c.contains("/releases/download/"))
        .count();
    assert_eq!(downloads, 3, "three download attempts before the fallback");
}

#[test]
fn release_asset_does_not_retry_a_missing_asset() {
    let (gh, calls) = github(|_, _, _| (404, Vec::new()));
    assert!(gh.release_asset("o/tool", "v1.0.0", "tool.tar.gz").is_err());
    assert_eq!(
        calls.lock().unwrap().len(),
        1,
        "a 404 must not be retried or sent to the API"
    );
}

#[test]
fn latest_tag_falls_back_to_the_api() {
    let (gh, _) = github(|_, path, _| match path {
        "/o/tool/releases/latest" => (504, b"/o/tool/releases/latest".to_vec()),
        "/repos/o/tool/releases/latest" => (200, br#"{"tag_name":"v2.0.0"}"#.to_vec()),
        _ => (404, Vec::new()),
    });
    assert_eq!(gh.latest_tag("o/tool").unwrap(), "v2.0.0");
}

#[test]
fn latest_tag_follows_the_redirect_and_names_a_missing_release() {
    let (gh, _) = github(|_, path, _| match path {
        "/o/tool/releases/latest" => (200, b"/o/tool/releases/tag/v1.4.0".to_vec()),
        _ => (404, b"/o/none/releases".to_vec()),
    });
    assert_eq!(gh.latest_tag("o/tool").unwrap(), "v1.4.0");
    let err = gh.latest_tag("o/none").unwrap_err().to_string();
    assert!(
        err.contains("public") && err.contains("release"),
        "got {err}"
    );
}

#[test]
fn ignore_repo_mise_adds_the_checkout() {
    let env = ignore_repo_mise(Some("/already".into()), Path::new("/work/src"));
    assert_eq!(env, std::ffi::OsString::from("/already:/work/src"));
    assert_eq!(
        ignore_repo_mise(None, Path::new("/work/src")),
        std::ffi::OsString::from("/work/src")
    );
}

#[test]
fn api_headers_send_the_token_only_when_present() {
    let (gh, _) = github(|_, _, _| (404, Vec::new()));
    assert!(!gh
        .api_headers("x")
        .iter()
        .any(|(k, _)| *k == "Authorization"));
    let mut gh = gh;
    gh = super::super::Github::new(gh.net, Box::new(|| "t0k".to_string()));
    assert!(gh
        .api_headers("x")
        .contains(&("Authorization", "Bearer t0k".to_string())));
}

#[test]
fn latest_resolves_tags_concurrently() {
    let (gh, _) = github(|_, path, _| match path {
        "/FacileStudio/filet/releases/latest" => (200, b"/x/releases/tag/v1.2.3".to_vec()),
        "/FacileStudio/sonar/releases/latest" => (200, b"/x/releases/tag/v0.10.0".to_vec()),
        _ => (404, Vec::new()),
    });
    let got = gh.latest(&["FacileStudio/filet".into(), "FacileStudio/sonar".into()]);
    assert_eq!(got["FacileStudio/filet"], "v1.2.3");
    assert_eq!(got["FacileStudio/sonar"], "v0.10.0");
    assert_eq!(got.len(), 2);
}

#[test]
fn latest_handles_an_empty_repo_list() {
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = hits.clone();
    let (gh, _) = github(move |_, _, _| {
        seen.fetch_add(1, Ordering::SeqCst);
        (404, Vec::new())
    });
    assert!(gh.latest(&[]).is_empty());
    assert_eq!(hits.load(Ordering::SeqCst), 0);
}
