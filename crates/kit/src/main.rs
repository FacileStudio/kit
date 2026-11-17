//! kit: the Facile package manager.

mod cli;
mod cmd;
mod ui;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
    let ui = ui::Ui::new(ui::Mode {
        json: cli.global.json,
        quiet: cli.global.quiet,
        no_color: cli.global.no_color,
        verbose: cli.global.verbose,
    });
    let code = match cmd::run(&cli, &ui) {
        Ok(()) => 0,
        Err(err) => ui::report(&ui, &err),
    };
    std::process::exit(code);
}
