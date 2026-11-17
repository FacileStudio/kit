use std::fs;
use std::time::{Duration, SystemTime};

use super::super::{check_bin_dir, check_copies, check_shadow, check_staged, LatestCache};
use super::{github, stub_binary};

fn redirect(_: &str, path: &str, _: &[(&str, String)]) -> (u16, Vec<u8>) {
    match path {
        "/o/a/releases/latest" => (200, b"/o/a/releases/tag/v2.0.0".to_vec()),
        _ => (404, Vec::new()),
    }
}

#[test]
fn latest_cache_answers_fresh_entries_without_the_network() {
    let dir = tempfile::tempdir().unwrap();
    let cache = LatestCache::in_dir(dir.path().to_path_buf());
    let now = SystemTime::now();
    let (gh, calls) = github(redirect);
    assert_eq!(cache.resolve(&gh, &["o/a".into()], now)["o/a"], "v2.0.0");
    assert_eq!(cache.resolve(&gh, &["o/a".into()], now)["o/a"], "v2.0.0");
    assert_eq!(
        calls.lock().unwrap().len(),
        1,
        "the second read must come from latest.json"
    );
    assert!(dir.path().join("latest.json").exists());
}

#[test]
fn latest_cache_refreshes_after_a_day_and_never_caches_a_failure() {
    let dir = tempfile::tempdir().unwrap();
    let cache = LatestCache::in_dir(dir.path().to_path_buf());
    let now = SystemTime::now();
    let (gh, calls) = github(redirect);
    cache.resolve(&gh, &["o/a".into(), "o/missing".into()], now);
    let later = now + Duration::from_secs(25 * 60 * 60);
    let got = cache.resolve(&gh, &["o/a".into(), "o/missing".into()], later);
    assert_eq!(got["o/missing"], "");
    let heads = calls
        .lock()
        .unwrap()
        .iter()
        .filter(|c| c.starts_with("HEAD /o/a/"))
        .count();
    assert_eq!(heads, 2, "a day-old entry must be asked again");
}

#[test]
fn doctor_reports_a_bin_dir_missing_from_path() {
    let home = tempfile::tempdir().unwrap();
    let bin = home.path().join(".local/bin");
    assert!(check_bin_dir(&bin, "/usr/bin".as_ref(), home.path())
        .unwrap()
        .message
        .contains("does not exist"));
    fs::create_dir_all(&bin).unwrap();
    let found = check_bin_dir(&bin, "/usr/bin".as_ref(), home.path()).unwrap();
    assert_eq!(found.message, "~/.local/bin is not on your PATH");
    assert_eq!(
        found.hints,
        [format!("export PATH=\"{}:$PATH\"", bin.display())]
    );
    let path = std::env::join_paths([bin.as_path()]).unwrap();
    assert!(check_bin_dir(&bin, &path, home.path()).is_none());
}

#[test]
fn doctor_reports_a_shadowed_binary_and_stray_copies() {
    let root = tempfile::tempdir().unwrap();
    let bin = root.path().join("bin");
    let other = root.path().join("other");
    stub_binary(&bin, "opus", "opus 1.0.0");
    stub_binary(&other, "opus", "opus 0.1.0");
    let path = std::env::join_paths([&other, &bin]).unwrap();
    assert!(check_shadow("opus", &bin, &path)
        .unwrap()
        .message
        .contains("comes first"));
    let path_ok = std::env::join_paths([&bin, &other]).unwrap();
    assert!(check_shadow("opus", &bin, &path_ok).is_none());
    let copies = check_copies("opus", &bin.join("opus"), &path).unwrap();
    assert_eq!(copies.hints, [other.join("opus").display().to_string()]);
}

#[test]
fn doctor_reports_interrupted_installs() {
    let dir = tempfile::tempdir().unwrap();
    assert!(check_staged(dir.path(), dir.path()).is_none());
    fs::write(dir.path().join(".opus.new.123"), "").unwrap();
    fs::write(dir.path().join("opus"), "").unwrap();
    let found = check_staged(dir.path(), dir.path()).unwrap();
    assert_eq!(
        found.hints,
        [format!("rm {}", dir.path().join(".opus.new.123").display())]
    );
}
