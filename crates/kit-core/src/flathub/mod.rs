//! The Flathub backend: Flatpak apps at user scope, driven through the `flatpak` CLI.

mod error;
mod id;
mod locate;
mod run;

#[cfg(all(test, unix))]
mod cli_tests;
#[cfg(test)]
mod id_tests;
#[cfg(all(test, unix))]
mod stub;

use std::path::PathBuf;

pub use error::FlathubError;
pub use id::validate_id;

/// The remote name kit installs from.
pub const REMOTE: &str = "flathub";
/// The Flathub repository description `ensure_remote` adds.
pub const REMOTE_URL: &str = "https://dl.flathub.org/repo/flathub.flatpakrepo";

const NON_INTERACTIVE: [&str; 3] = ["--user", "-y", "--noninteractive"];

/// A handle on one `flatpak` binary. Every call works on the user installation.
#[derive(Debug, Clone)]
pub struct Flatpak {
    bin: PathBuf,
}

impl Flatpak {
    /// Adds the Flathub remote to the user installation unless it is already there.
    pub fn ensure_remote(&self) -> Result<(), FlathubError> {
        run::run(
            &self.bin,
            &[
                "remote-add",
                "--user",
                "--if-not-exists",
                REMOTE,
                REMOTE_URL,
            ],
        )
        .map(drop)
    }

    /// Installs apps from Flathub. Validates every ID first and does nothing for an empty list.
    pub fn install(&self, ids: &[&str]) -> Result<(), FlathubError> {
        self.apply("install", Some(REMOTE), ids)
    }

    /// Uninstalls apps. Validates every ID first and does nothing for an empty list.
    pub fn uninstall(&self, ids: &[&str]) -> Result<(), FlathubError> {
        self.apply("uninstall", None, ids)
    }

    /// Updates every app and runtime in the user installation.
    pub fn update(&self) -> Result<(), FlathubError> {
        let mut args = vec!["update"];
        args.extend(NON_INTERACTIVE);
        run::run(&self.bin, &args).map(drop)
    }

    /// The IDs of the apps in the user installation.
    pub fn list_installed(&self) -> Result<Vec<String>, FlathubError> {
        let out = run::run(
            &self.bin,
            &["list", "--user", "--app", "--columns=application"],
        )?;
        Ok(out
            .lines()
            .map(str::trim)
            .filter(|id| validate_id(id).is_ok())
            .map(String::from)
            .collect())
    }

    /// Searches the remotes and returns `(id, name)` pairs.
    pub fn search(&self, query: &str) -> Result<Vec<(String, String)>, FlathubError> {
        if query.trim().is_empty() {
            return Err(FlathubError::EmptyQuery);
        }
        let out = run::run(
            &self.bin,
            &[
                "search",
                "--user",
                "--columns=application,name",
                "--",
                query,
            ],
        )?;
        Ok(out.lines().filter_map(run::parse_search_line).collect())
    }

    fn apply(&self, action: &str, remote: Option<&str>, ids: &[&str]) -> Result<(), FlathubError> {
        ids.iter().try_for_each(|id| validate_id(id))?;
        if ids.is_empty() {
            return Ok(());
        }
        let mut args = vec![action];
        args.extend(NON_INTERACTIVE);
        args.extend(remote);
        args.extend(ids);
        run::run(&self.bin, &args).map(drop)
    }
}
