//! The arguments of each command.

use clap::{Args, Subcommand};

/// `kit add`.
#[derive(Debug, Clone, Default, Args)]
#[command(long_about = "Add packages to the list file and install them.\n\n\
A package is a bare name (sablier, firefox, ripgrep) or backend:name (facile:sablier, \
flathub:org.mozilla.firefox, nix:ripgrep). A bare name is looked up in the Facile catalog, \
then on Flathub, then in nixpkgs. Separate several with spaces or commas.\n\n\
With no arguments it opens a picker over the Facile catalog. Pass --all to take every Facile tool.")]
pub struct AddArgs {
    /// Packages to add
    #[arg(value_name = "package")]
    pub specs: Vec<String>,
    /// Add every tool in the Facile catalog
    #[arg(long)]
    pub all: bool,
    /// Build Facile tools from source, ignore published releases
    #[arg(long)]
    pub source: bool,
    /// Release tag of the Facile tool to install (default latest)
    #[arg(long = "version", id = "tag", value_name = "tag")]
    pub tag: Option<String>,
    /// Skip AI agent skill registration
    #[arg(long)]
    pub no_skill: bool,
    /// Skip MCP server registration in agent harnesses
    #[arg(long)]
    pub no_mcp: bool,
    /// Replace a binary another package manager placed in the bin directory
    #[arg(long)]
    pub force: bool,
    /// Never prompt: fail when a name matches several packages
    #[arg(short, long)]
    pub yes: bool,
}

/// `kit del`.
#[derive(Debug, Clone, Default, Args)]
#[command(
    long_about = "Remove packages from the list file and uninstall them.\n\n\
Nix packages are removed by rebuilding the kit Nix profile without them."
)]
pub struct DelArgs {
    /// Packages to remove
    #[arg(value_name = "package", required = true)]
    pub specs: Vec<String>,
}

/// `kit list`.
#[derive(Debug, Clone, Default, Args)]
pub struct ListArgs {
    /// Ask GitHub for the latest Facile releases now instead of using the daily cache
    #[arg(long)]
    pub check: bool,
}

/// Commands that ask before they change anything.
#[derive(Debug, Clone, Default, Args)]
pub struct YesArgs {
    /// Do not ask for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

/// `kit update`.
#[derive(Debug, Clone, Default, Args)]
#[command(long_about = "Bring listed packages up to date.\n\n\
With no arguments it updates every listed Facile tool, every Flathub app, the Nix profile \
and kit itself, last. A Facile tool that already reports its latest release is left alone; \
pass --force to reinstall it anyway.")]
pub struct UpdateArgs {
    /// Packages to update (default everything listed, and kit)
    #[arg(value_name = "package")]
    pub names: Vec<String>,
    /// Reinstall even when already at the latest release
    #[arg(long)]
    pub force: bool,
    /// Report what is out of date without installing anything
    #[arg(long)]
    pub check: bool,
    /// Build Facile tools from source, ignore published releases
    #[arg(long)]
    pub source: bool,
    /// Skip AI agent skill registration
    #[arg(long)]
    pub no_skill: bool,
}

/// `kit undo`.
#[derive(Debug, Clone, Default, Args)]
pub struct UndoArgs {
    /// Source to undo: only nix keeps generations
    #[arg(value_name = "source", default_value = "nix")]
    pub source: String,
}

/// `kit setup`.
#[derive(Debug, Args)]
pub struct SetupArgs {
    /// The backend to set up.
    #[command(subcommand)]
    pub target: SetupTarget,
}

/// What `kit setup` can set up.
#[derive(Debug, Subcommand)]
pub enum SetupTarget {
    /// Install Nix with the upstream multi-user installer
    Nix(YesArgs),
}

/// `kit self-update`.
#[derive(Debug, Clone, Copy, Default, Args)]
pub struct SelfArgs {
    /// Replace kit even when it is current or a source build
    #[arg(long)]
    pub force: bool,
    /// Build kit from source, ignore published releases
    #[arg(long)]
    pub source: bool,
}
