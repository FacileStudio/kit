use super::net::HttpError;
use super::owner::Owner;

/// Everything that can stop a Facile tool install, update or removal.
#[derive(Debug, thiserror::Error)]
pub enum FacileError {
    /// A release artifact downloaded but did not verify. Never retried as a source build.
    #[error("{0}")]
    Integrity(String),
    /// The repository has no readable release.
    #[error("{0}")]
    NoRelease(String),
    /// A request failed.
    #[error(transparent)]
    Http(#[from] HttpError),
    /// A download failed, then its API fallback failed too.
    #[error("{0}")]
    Download(String),
    /// A source build could not run or did not produce the binaries.
    #[error("{0}")]
    Build(String),
    /// The binary could not be placed, or did not run once placed.
    #[error("{0}")]
    Install(String),
    /// Another package manager owns the binary, so kit leaves it alone.
    #[error("{name} is managed by {owner}, run `{command}` instead")]
    Foreign {
        /// The binary's name.
        name: String,
        /// Who installed it.
        owner: Owner,
        /// The command that updates it.
        command: String,
    },
    /// A file operation failed.
    #[error("{context}: {source}")]
    Io {
        /// What kit was doing.
        context: String,
        /// The underlying error.
        source: std::io::Error,
    },
}

impl FacileError {
    /// True for a failure that must abort rather than fall back to a source build.
    pub fn is_integrity(&self) -> bool {
        matches!(self, FacileError::Integrity(_))
    }

    pub(crate) fn io(context: impl Into<String>) -> impl FnOnce(std::io::Error) -> FacileError {
        let context = context.into();
        move |source| FacileError::Io { context, source }
    }
}
