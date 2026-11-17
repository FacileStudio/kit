use std::fmt;

use serde_json::{json, Value};

use super::Ui;

/// An error that carries its exit code, hints and JSON form. A silent one was already printed.
#[derive(Debug, Default)]
pub struct Failure {
    /// The exit status.
    pub code: i32,
    /// The error line, or `None` when the command already reported.
    pub message: Option<String>,
    /// Lines that say what to do.
    pub hints: Vec<String>,
    /// Extra fields for the `--json` error document.
    pub extra: Option<Value>,
}

impl Failure {
    /// Exit with `code` and print nothing more.
    pub fn silent(code: i32) -> Failure {
        Failure {
            code,
            ..Failure::default()
        }
    }

    /// Exit 1 with `message` and `hints`.
    pub fn new(message: impl Into<String>, hints: Vec<String>) -> Failure {
        Failure {
            code: 1,
            message: Some(message.into()),
            hints,
            extra: None,
        }
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message.as_deref().unwrap_or("failed"))
    }
}

impl std::error::Error for Failure {}

/// Prints a command's error the way the mode asks and returns the exit status.
pub fn report(ui: &Ui, err: &anyhow::Error) -> i32 {
    let fallback;
    let failure = match err.downcast_ref::<Failure>() {
        Some(f) => f,
        None => {
            fallback = Failure::new(format!("{err:#}"), Vec::new());
            &fallback
        }
    };
    let Some(message) = &failure.message else {
        return failure.code;
    };
    if ui.json() {
        let mut doc = json!({ "error": message, "hints": failure.hints });
        if let (Some(Value::Object(extra)), Some(map)) = (&failure.extra, doc.as_object_mut()) {
            map.extend(extra.clone());
        }
        ui.emit(&doc);
        return failure.code;
    }
    ui.error(message);
    failure.hints.iter().for_each(|h| ui.hint(h));
    failure.code
}
