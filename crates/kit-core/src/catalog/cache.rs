use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// How long a fetched catalog or list stays fresh.
pub(crate) const MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// True when the file is under a day old and was written after the running binary.
///
/// A new kit carries a new embedded catalog, so a cache from the old binary must not hide it.
pub(crate) fn fresh(path: &Path) -> bool {
    fresh_against(path, current_exe_mtime())
}

/// The freshness rule with the binary's modification time passed in.
pub(crate) fn fresh_against(path: &Path, binary: Option<SystemTime>) -> bool {
    let Some(written) = mtime(path) else {
        return false;
    };
    if younger_than(written, MAX_AGE).is_none() {
        return false;
    }
    binary.is_none_or(|b| b <= written)
}

/// The modification time of a file, when it exists.
pub(crate) fn mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// `Some(())` when `when` is less than `age` ago.
pub(crate) fn younger_than(when: SystemTime, age: Duration) -> Option<()> {
    match SystemTime::now().duration_since(when) {
        Ok(elapsed) if elapsed >= age => None,
        _ => Some(()),
    }
}

/// Writes a cache file, creating its directory. A failure only costs a refetch, so it is ignored.
pub(crate) fn write(path: &Path, raw: &[u8]) {
    if let Some(dir) = path.parent() {
        if fs::create_dir_all(dir).is_err() {
            return;
        }
    }
    let _ = fs::write(path, raw);
}

fn current_exe_mtime() -> Option<SystemTime> {
    std::env::current_exe().ok().and_then(|p| mtime(&p))
}
