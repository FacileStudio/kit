//! One file per command, plus the pieces they share.

mod add;
mod apply;
mod backends;
mod check;
mod clean;
mod ctx;
mod del;
mod doctor;
mod doctor_more;
mod drift;
mod dump;
mod list;
mod list_render;
mod machine;
mod names;
mod pick;
mod selfupdate;
mod setup;
mod sync;
mod undo;
mod update;
mod update_parts;
mod version;

#[cfg(test)]
mod tests;

use anyhow::Result;

use crate::cli::{Cli, Command};
use crate::ui::Ui;
pub use ctx::Ctx;

/// The version compiled into this binary.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Runs the parsed command line.
pub fn run(cli: &Cli, ui: &Ui) -> Result<()> {
    let ctx = Ctx::new(&cli.global, ui)?;
    match &cli.command {
        Command::Add(args) => add::run(&ctx, args),
        Command::Del(args) => del::run(&ctx, args),
        Command::Sync => sync::run(&ctx),
        Command::List(args) => list::run(&ctx, args),
        Command::Check => check::run(&ctx),
        Command::Dump(args) => dump::run(&ctx, args.yes),
        Command::Clean(args) => clean::run(&ctx, args.yes),
        Command::Update(args) => update::run(&ctx, args),
        Command::Undo(args) => undo::run(&ctx, &args.source),
        Command::Doctor => doctor::run(&ctx),
        Command::Setup(args) => setup::run(&ctx, &args.target),
        Command::SelfUpdate(args) => selfupdate::run(&ctx, *args),
        Command::Version => version::run(&ctx),
    }
}
