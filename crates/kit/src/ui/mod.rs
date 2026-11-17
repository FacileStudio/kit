//! Runtime output: the five levels of standards/cli section 7, colour discipline and prompts.

mod failure;
mod print;
mod prompt;
mod reporter;
pub mod style;

use std::io::IsTerminal;
use std::sync::atomic::AtomicBool;

pub use failure::{report, Failure};

/// Where output goes and how it looks, decided once from the flags and the terminal.
#[derive(Debug, Default)]
pub struct Ui {
    out_color: bool,
    err_color: bool,
    json: bool,
    quiet: bool,
    verbose: u8,
    hint_to_err: AtomicBool,
    hint_hidden: AtomicBool,
}

/// The flags that shape output.
#[derive(Debug, Clone, Copy, Default)]
pub struct Mode {
    /// `--json`: one JSON document on stdout, no colour, no prompts.
    pub json: bool,
    /// `--quiet`: nothing on stdout except data.
    pub quiet: bool,
    /// `--no-color`.
    pub no_color: bool,
    /// `-v` count.
    pub verbose: u8,
}

impl Ui {
    /// A printer for `mode`. Colour needs a TTY, no `NO_COLOR`, no `--no-color` and no `--json`.
    pub fn new(mode: Mode) -> Ui {
        let allowed = !mode.no_color
            && !mode.json
            && std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty());
        Ui {
            out_color: allowed && std::io::stdout().is_terminal(),
            err_color: allowed && std::io::stderr().is_terminal(),
            json: mode.json,
            quiet: mode.quiet,
            verbose: mode.verbose,
            ..Ui::default()
        }
    }

    /// True when `--json` was passed.
    pub fn json(&self) -> bool {
        self.json
    }

    /// True when `--quiet` was passed.
    pub fn quiet(&self) -> bool {
        self.quiet
    }

    /// True when kit may ask a question: a terminal on stdin and stderr, and no `--json`.
    pub fn interactive(&self) -> bool {
        !self.json && std::io::stdin().is_terminal() && std::io::stderr().is_terminal()
    }

    /// A detail line on stderr, shown with `-v`.
    pub fn debug(&self, message: &str) {
        if self.verbose > 0 {
            eprintln!("{}", self.dim_err(message));
        }
    }
}
