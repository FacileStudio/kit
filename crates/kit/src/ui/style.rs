use super::Ui;

/// The colours of standards/cli section 7.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Cyan: steps, and the field to look at first.
    Accent,
    /// Green: success and healthy values.
    Good,
    /// Yellow: warnings and values that are not releases.
    Notice,
    /// Red: errors and broken values.
    Alert,
    /// Faint: hints and secondary text.
    Dim,
}

impl Tone {
    fn code(self) -> &'static str {
        match self {
            Tone::Accent => "36",
            Tone::Good => "32",
            Tone::Notice => "33",
            Tone::Alert => "31",
            Tone::Dim => "2",
        }
    }
}

impl Ui {
    /// `text` in `tone` when stdout takes colour.
    pub fn tint(&self, tone: Tone, text: &str) -> String {
        self.paint(self.out_color, tone, text)
    }

    /// `text` faint when stdout takes colour.
    pub fn dim(&self, text: &str) -> String {
        self.paint(self.out_color, Tone::Dim, text)
    }

    /// `text` faint when stderr takes colour.
    pub fn dim_err(&self, text: &str) -> String {
        self.paint(self.err_color, Tone::Dim, text)
    }

    pub(super) fn paint(&self, on: bool, tone: Tone, text: &str) -> String {
        if on {
            format!("\x1b[{}m{text}\x1b[0m", tone.code())
        } else {
            text.to_string()
        }
    }
}
