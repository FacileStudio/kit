use std::path::{Path, PathBuf};

use super::archive::{extract, verify_checksum};
use super::{Backend, FacileError, Reporter};
use crate::catalog::Tool;

/// The `<os>_<arch>` pair release archives are named with.
pub fn platform() -> (&'static str, &'static str) {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => other,
    };
    (os, arch)
}

/// The archive name for a tool at a version: `<asset>_<version>_<os>_<arch>.tar.gz`.
pub fn archive_name(asset: &str, version: &str) -> String {
    let (os, arch) = platform();
    format!("{asset}_{version}_{os}_{arch}.tar.gz")
}

impl Backend {
    /// Downloads the release archive, verifies it against `checksums.txt`, and extracts every binary.
    pub(crate) fn download_release(
        &self,
        tool: &Tool,
        version: &str,
        work: &Path,
        r: &dyn Reporter,
    ) -> Result<Vec<PathBuf>, FacileError> {
        let tag = if version.is_empty() {
            self.github.latest_tag(&tool.repo)?
        } else {
            version.to_string()
        };
        let ver = tag.trim_start_matches('v');
        let archive = archive_name(&tool.asset, ver);
        let (os, arch) = platform();
        r.step(&format!("Downloading {} {ver} for {os}/{arch}", tool.bin));
        let blob = self.github.release_asset(&tool.repo, &tag, &archive)?;
        let sums = self
            .github
            .release_asset(&tool.repo, &tag, "checksums.txt")?;
        verify_checksum(&blob, &archive, &String::from_utf8_lossy(&sums))?;
        tool.bin_list()
            .iter()
            .map(|bin| extract(&blob, bin, work))
            .collect()
    }
}
