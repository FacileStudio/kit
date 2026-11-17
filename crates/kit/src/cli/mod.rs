//! The command line: global flags and the command tree, in one place.

mod args;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub use args::{
    AddArgs, DelArgs, ListArgs, SelfArgs, SetupArgs, SetupTarget, UndoArgs, UpdateArgs, YesArgs,
};

/// Keep one list of Facile tools, Flathub apps and Nix packages, and make the machine match it.
#[derive(Debug, Parser)]
#[command(
    name = "kit",
    version,
    about = "Install and manage Facile tools, Flathub apps and Nix packages"
)]
#[command(
    long_about = "kit keeps one list file of Facile tools, Flathub apps and Nix packages, \
and makes this machine match it.\n\nThe list lives in ~/.config/kit/kit.yml unless --file or KIT_FILE says otherwise."
)]
pub struct Cli {
    /// Flags every command accepts.
    #[command(flatten)]
    pub global: Global,
    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Flags every command accepts.
#[derive(Debug, Clone, Default, clap::Args)]
pub struct Global {
    /// List file to read and write (default ~/.config/kit/kit.yml)
    #[arg(long, global = true, value_name = "path")]
    pub file: Option<PathBuf>,
    /// Directory Facile tools install into (default ~/.local/bin)
    #[arg(long, global = true, value_name = "dir")]
    pub bin_dir: Option<PathBuf>,
    /// Print one JSON document to stdout
    #[arg(long, global = true)]
    pub json: bool,
    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
    /// Print errors and warnings only
    #[arg(short, long, global = true)]
    pub quiet: bool,
    /// Print more detail
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

/// Every kit command.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Add packages to the list and install them
    Add(AddArgs),
    /// Remove packages from the list and uninstall them
    #[command(visible_aliases = ["rm", "remove", "uninstall"])]
    Del(DelArgs),
    /// Install what the list names and is missing
    Sync,
    /// List what the list file names and what is installed
    List(ListArgs),
    /// Report drift between the list file and this machine
    Check,
    /// Add what is installed but not listed to the list file
    Dump(YesArgs),
    /// Remove what is installed but not listed
    Clean(YesArgs),
    /// Update listed packages and kit itself
    Update(UpdateArgs),
    /// Roll the Nix profile back one generation
    Undo(UndoArgs),
    /// Check the health of the kit installation
    Doctor,
    /// Set up a package backend
    Setup(SetupArgs),
    /// Replace kit with its latest release
    SelfUpdate(SelfArgs),
    /// Print the kit version
    Version,
}
