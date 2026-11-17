use std::fs::OpenOptions;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::FacileError;

const DOWNLOAD_LIMIT: u64 = 256 << 20;

/// Checks `blob` against its line in `checksums.txt`. A mismatch or a missing line is an integrity failure.
pub(crate) fn verify_checksum(blob: &[u8], name: &str, sums: &str) -> Result<(), FacileError> {
    let want = hex::encode(Sha256::digest(blob));
    for line in sums.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() == 2 && fields[1].trim_start_matches('*') == name {
            if fields[0] == want {
                return Ok(());
            }
            return Err(FacileError::Integrity(format!(
                "checksum mismatch for {name}: re-cut the release, this artifact is not the one that was published"
            )));
        }
    }
    Err(FacileError::Integrity(format!(
        "{name} is not listed in checksums.txt: re-cut the release"
    )))
}

/// Writes the regular file called `bin` from a `.tar.gz` into `work` and returns its path.
pub(crate) fn extract(blob: &[u8], bin: &str, work: &Path) -> Result<PathBuf, FacileError> {
    let unreadable = |e: io::Error| FacileError::Install(format!("cannot read the archive: {e}"));
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(blob));
    for entry in archive.entries().map_err(unreadable)? {
        let mut entry = entry.map_err(unreadable)?;
        if !is_binary_entry(&entry, bin) {
            continue;
        }
        let dest = work.join(bin);
        write_from(&mut entry, &dest)?;
        return Ok(dest);
    }
    Err(FacileError::Install(format!(
        "the archive contains no {bin} binary"
    )))
}

fn is_binary_entry<R: Read>(entry: &tar::Entry<R>, bin: &str) -> bool {
    entry.header().entry_type() == tar::EntryType::Regular
        && entry
            .path()
            .ok()
            .and_then(|p| p.file_name().map(|n| n == bin))
            .unwrap_or(false)
}

fn write_from(reader: &mut impl Read, dest: &Path) -> Result<(), FacileError> {
    let mut options = OpenOptions::new();
    options.create(true).write(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o755);
    let mut file = options
        .open(dest)
        .map_err(FacileError::io(format!("cannot write {}", dest.display())))?;
    io::copy(&mut reader.take(DOWNLOAD_LIMIT), &mut file)
        .map_err(FacileError::io(format!("cannot write {}", dest.display())))?;
    Ok(())
}
