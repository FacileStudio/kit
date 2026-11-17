//! The errors the Flathub backend returns.

/// Why a Flathub operation failed.
#[derive(Debug, thiserror::Error)]
pub enum FlathubError {
    /// No `flatpak` binary on PATH, or the configured one is gone.
    #[error("flatpak is not available: install it with your distribution's package manager")]
    NotAvailable,
    /// An app ID that is not a valid Flatpak name.
    #[error("invalid Flatpak app ID {id:?}: {reason}")]
    InvalidId {
        /// The rejected ID.
        id: String,
        /// What rule it breaks.
        reason: &'static str,
    },
    /// An empty search query.
    #[error("the search query is empty")]
    EmptyQuery,
    /// flatpak ran and exited non-zero.
    #[error("flatpak {action} failed ({}): {stderr}", exit_label(*.code))]
    CommandFailed {
        /// The flatpak subcommand.
        action: String,
        /// The exit code, or `None` when a signal killed it.
        code: Option<i32>,
        /// The last lines flatpak wrote to stderr.
        stderr: String,
    },
    /// flatpak could not be started or its output could not be read.
    #[error("cannot run flatpak: {0}")]
    Io(#[from] std::io::Error),
}

fn exit_label(code: Option<i32>) -> String {
    code.map_or_else(
        || "killed by a signal".to_string(),
        |code| format!("exit code {code}"),
    )
}
