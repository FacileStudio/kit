use std::io;

/// Everything the Nix backend can fail with.
#[derive(Debug, thiserror::Error)]
pub enum NixError {
    /// A Nix binary is not on PATH.
    #[error("{0} is not installed: run `kit setup nix`")]
    NotAvailable(String),
    /// A Nix command exited non-zero.
    #[error("`{command}` failed: {stderr}")]
    CommandFailed {
        /// The command that failed, without its arguments.
        command: String,
        /// The last lines of its stderr.
        stderr: String,
    },
    /// An attribute name kit refuses to put in a Nix expression.
    #[error("invalid nix attribute {0:?}: use letters, digits, '-', '_' and '.' between names")]
    InvalidAttr(String),
    /// Nix printed something kit cannot read.
    #[error("unexpected output from nix: {0}")]
    BadOutput(String),
    /// Reading or writing kit's state failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// The state file is not valid JSON.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
