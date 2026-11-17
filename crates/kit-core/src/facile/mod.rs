//! The Facile tool backend: release download, checksum, atomic install, source build, agent skills,
//! MCP registration, update checks, self-update and doctor checks.
//!
//! Load-bearing rules, kept from facile:
//! - Verify by running: every version reported is the one the installed binary printed.
//! - A checksum mismatch aborts and never falls back to a source build.
//! - Install stages beside the destination, then renames.
//! - No state file: what is installed is discovered by running the binaries.
//! - `update` compares the binary's version with the live latest tag before it downloads.
//! - PATH problems are reported, never fixed: kit does not edit shell configuration.
//! - Never sudo, never write outside the bin directory and kit's own directories.

mod archive;
mod backend;
mod build;
mod doctor;
mod error;
mod github;
mod inject;
mod install;
mod latest;
mod latest_cache;
mod net;
mod owner;
mod pathutil;
mod place;
mod release;
mod report;
mod runlog;
mod selfupdate;
mod skill;
mod source;
mod status;
mod update;
mod uptodate;
mod version;

#[cfg(all(test, unix))]
mod tests;

pub use backend::{Backend, Env};
pub use doctor::{check_bin_dir, check_copies, check_shadow, check_staged, Finding};
pub use error::FacileError;
pub use github::{token_from_env, Github};
pub use install::InstallOptions;
pub use latest_cache::{LatestCache, LATEST_MAX_AGE};
pub use net::{Head, HttpError, Net, UreqNet};
pub use owner::{from_homebrew, owner_of, Owner};
pub use pathutil::{on_path, real_path, tilde, which};
pub use place::{installed_line, uninstall, verify};
pub use release::{archive_name, platform};
pub use report::{Quiet, Reporter};
pub use selfupdate::{self_dir, self_tool, SelfOptions, SelfOutcome, KIT_REPO};
pub use status::{installed_tool, InstalledTool, ToolStatus};
pub use update::{UpdateOptions, UpdateOutcome};
pub use uptodate::{matches_tag, stale, up_to_date, TagFn};
pub use version::{is_semver, outdated, version_of};
