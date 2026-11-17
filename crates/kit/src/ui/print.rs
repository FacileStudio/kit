use std::io::Write;
use std::sync::atomic::Ordering;

use serde::Serialize;

use super::style::Tone;
use super::Ui;

impl Ui {
    /// Work about to happen, on stdout.
    pub fn step(&self, message: &str) {
        self.line(false, Tone::Accent, "▸", message);
    }

    /// Work that completed, on stdout.
    pub fn success(&self, message: &str) {
        self.line(false, Tone::Good, "✓", message);
    }

    /// A degraded state the run continues through, on stderr.
    pub fn warn(&self, message: &str) {
        self.line(true, Tone::Notice, "!", message);
    }

    /// The failure a run aborts on, on stderr.
    pub fn error(&self, message: &str) {
        self.line(true, Tone::Alert, "✗", message);
    }

    /// An indented dim line under the line it explains, on that line's stream.
    pub fn hint(&self, message: &str) {
        if self.hint_hidden.load(Ordering::Relaxed) {
            return;
        }
        if self.hint_to_err.load(Ordering::Relaxed) {
            eprintln!("  {}", self.dim_err(message));
        } else {
            println!("  {}", self.dim(message));
        }
    }

    /// A data line on stdout with no glyph. Suppressed under `--json`.
    pub fn out(&self, text: &str) {
        if !self.json {
            println!("{text}");
        }
    }

    /// The one JSON document a `--json` run prints.
    pub fn emit<T: Serialize>(&self, value: &T) {
        let mut stdout = std::io::stdout().lock();
        if serde_json::to_writer_pretty(&mut stdout, value).is_ok() {
            let _ = writeln!(stdout);
        }
    }

    fn line(&self, to_err: bool, tone: Tone, glyph: &str, message: &str) {
        let hidden = !to_err && (self.json || self.quiet);
        self.hint_to_err.store(to_err, Ordering::Relaxed);
        self.hint_hidden.store(hidden, Ordering::Relaxed);
        if hidden {
            return;
        }
        if to_err {
            eprintln!("{} {message}", self.paint(self.err_color, tone, glyph));
        } else {
            println!("{} {message}", self.paint(self.out_color, tone, glyph));
        }
    }
}
