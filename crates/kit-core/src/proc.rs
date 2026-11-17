//! Spawning child processes.

use std::process::{Command, Output};
use std::time::Duration;

const TEXT_FILE_BUSY: i32 = 26;
const BUSY_RETRIES: usize = 50;
const BUSY_WAIT: Duration = Duration::from_millis(10);

/// Runs a command, retrying briefly while the executable is busy (`ETXTBSY`).
///
/// A binary written a moment ago can still be open for writing in a child another thread forked.
pub(crate) fn output(cmd: &mut Command) -> std::io::Result<Output> {
    for _ in 0..BUSY_RETRIES {
        match cmd.output() {
            Err(e) if e.raw_os_error() == Some(TEXT_FILE_BUSY) => std::thread::sleep(BUSY_WAIT),
            other => return other,
        }
    }
    cmd.output()
}
