//! The three backends a list entry can belong to, and the `backend:name` prefix.

use std::fmt;
use std::str::FromStr;

use super::ListError;

/// The section of the list file an entry lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Source {
    /// A Facile tool from the catalog.
    Facile,
    /// A Nix attribute.
    Nix,
    /// A Flathub application.
    Flathub,
}

impl Source {
    /// Every source, in list file order.
    pub const ALL: [Source; 3] = [Source::Facile, Source::Nix, Source::Flathub];

    /// The name used as a section key and as the `backend:` prefix.
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Facile => "facile",
            Source::Nix => "nix",
            Source::Flathub => "flathub",
        }
    }

    /// Splits `backend:name` into its source and name. A spec without a colon is a bare name.
    pub fn split_spec(spec: &str) -> Result<(Option<Source>, &str), ListError> {
        match spec.split_once(':') {
            Some((prefix, name)) => Ok((Some(prefix.parse()?), name)),
            None => Ok((None, spec)),
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Source {
    type Err = ListError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Source::ALL
            .into_iter()
            .find(|source| source.as_str() == s)
            .ok_or_else(|| ListError::UnknownSource(s.to_string()))
    }
}
