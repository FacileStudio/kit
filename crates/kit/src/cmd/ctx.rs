use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use kit_core::catalog::{Loader, Manifest};
use kit_core::facile::{tilde, Backend};
use kit_core::listfile::{resolve_path, KitFile};
use kit_core::paths;

use crate::cli::Global;
use crate::ui::Ui;

/// What every command needs: the printer, the list file and the bin directory.
pub struct Ctx<'a> {
    /// The printer.
    pub ui: &'a Ui,
    /// The list file.
    pub file: PathBuf,
    /// Where Facile tools go.
    pub bin_dir: PathBuf,
}

impl<'a> Ctx<'a> {
    /// Resolves the list file and bin directory from the flags and the environment.
    pub fn new(global: &Global, ui: &'a Ui) -> Result<Ctx<'a>> {
        let file = resolve_path(global.file.as_deref()).context("cannot locate the list file")?;
        let bin_dir = match &global.bin_dir {
            Some(dir) => dir.clone(),
            None => paths::bin_dir().context("cannot locate the bin directory")?,
        };
        ui.debug(&format!("list file {}", file.display()));
        Ok(Ctx { ui, file, bin_dir })
    }

    /// The list file as it is now.
    pub fn list(&self) -> Result<KitFile> {
        Ok(KitFile::read(&self.file)?)
    }

    /// Changes the list file under its lock.
    pub fn edit<T>(&self, change: impl FnOnce(&mut KitFile) -> T) -> Result<T> {
        Ok(KitFile::edit(&self.file, change)?)
    }

    /// The Facile catalog with the user's sources merged in.
    pub fn catalog(&self) -> Result<Manifest> {
        Ok(Loader::from_env()?.load()?)
    }

    /// The Facile tool backend on this machine.
    pub fn backend(&self) -> Result<Backend> {
        Backend::from_env().context("cannot read the environment")
    }

    /// `path` with the home directory shortened to `~`.
    pub fn tilde(&self, path: &Path) -> String {
        match paths::home() {
            Ok(home) => tilde(path, &home),
            Err(_) => path.display().to_string(),
        }
    }
}
