//! Turning what the user typed (`firefox`, `nix:ripgrep`) into a backend and an ID.
//!
//! This module only returns data. The CLI decides whether to prompt, print hints or fail.

mod bare;
#[cfg(test)]
mod error_tests;
#[cfg(test)]
mod fake;
#[cfg(test)]
mod tests;

use serde::{Serialize, Serializer};

use crate::catalog::{CatalogError, Tool};
use crate::flathub::{self, FlathubError};
use crate::listfile::{ListError, Source};
use crate::nix::{self, NixError};

/// Looks a tool up in the Facile catalog by name, case-insensitively.
pub type CatalogFn<'a> = dyn Fn(&str) -> Result<Option<Tool>, CatalogError> + 'a;
/// Searches Flathub and returns `(id, display name)` pairs.
pub type SearchFn<'a> = dyn Fn(&str) -> Result<Vec<(String, String)>, FlathubError> + 'a;
/// Reports whether a Nix attribute exists in nixpkgs.
pub type ExistsFn<'a> = dyn Fn(&str) -> Result<bool, NixError> + 'a;

/// The backends `resolve` consults. The CLI passes real ones; tests pass fakes.
pub struct Context<'a> {
    /// The catalog lookup.
    pub catalog: &'a CatalogFn<'a>,
    /// The Flathub search.
    pub flathub_search: &'a SearchFn<'a>,
    /// The Nix existence check.
    pub nix_exists: &'a ExistsFn<'a>,
    /// Whether an explicit `nix:attr` must also exist in nixpkgs. A bare name always checks.
    pub check_nix: bool,
}

/// One package a spec can mean.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Candidate {
    /// The backend.
    #[serde(serialize_with = "source_str")]
    pub source: Source,
    /// The ID to store in the list file: catalog name, Flathub app ID or Nix attribute.
    pub id: String,
    /// The human name: the catalog name, the Flathub display name or the Nix attribute.
    pub name: String,
}

/// What a spec resolved to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "result", rename_all = "lowercase")]
pub enum Resolution {
    /// Exactly one package.
    One(Candidate),
    /// Several packages from the same backend; the caller picks.
    Many(Vec<Candidate>),
    /// Nothing matched in the backends that were consulted.
    None,
}

/// The resolution plus the backends that could not be consulted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Outcome {
    /// The result.
    pub resolution: Resolution,
    /// Backends skipped because their tool (flatpak, nix) is not installed, in lookup order.
    #[serde(serialize_with = "sources_str")]
    pub skipped: Vec<Source>,
}

/// Why a spec could not be resolved.
#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    /// The spec or the name after the prefix is empty.
    #[error("the package name is empty")]
    Empty,
    /// The `backend:` prefix is not facile, nix or flathub.
    #[error(transparent)]
    Spec(#[from] ListError),
    /// `facile:name` names no catalog tool.
    #[error("no Facile tool named `{0}` in the catalog")]
    UnknownTool(String),
    /// `nix:attr` with the existence check on names no package.
    #[error("no nix package `{0}` in nixpkgs")]
    UnknownAttr(String),
    /// The catalog could not be loaded.
    #[error(transparent)]
    Catalog(#[from] CatalogError),
    /// Flathub rejected the ID or the search failed.
    #[error(transparent)]
    Flathub(#[from] FlathubError),
    /// Nix rejected the attribute or the check failed.
    #[error(transparent)]
    Nix(#[from] NixError),
}

/// Resolves `backend:name` directly after validation, or a bare name through Facile, Flathub, then Nix.
pub fn resolve(spec: &str, ctx: &Context<'_>) -> Result<Outcome, ResolveError> {
    let (source, name) = Source::split_spec(spec.trim())?;
    let name = name.trim();
    if name.is_empty() {
        return Err(ResolveError::Empty);
    }
    match source {
        Some(source) => prefixed(source, name, ctx).map(|one| Outcome {
            resolution: Resolution::One(one),
            skipped: Vec::new(),
        }),
        None => bare::resolve(name, ctx),
    }
}

impl Candidate {
    fn new(source: Source, id: &str, name: &str) -> Self {
        Self {
            source,
            id: id.to_string(),
            name: name.to_string(),
        }
    }
}

fn prefixed(source: Source, name: &str, ctx: &Context<'_>) -> Result<Candidate, ResolveError> {
    match source {
        Source::Facile => (ctx.catalog)(name)?
            .map(|tool| Candidate::new(Source::Facile, &tool.name, &tool.name))
            .ok_or_else(|| ResolveError::UnknownTool(name.to_string())),
        Source::Flathub => {
            flathub::validate_id(name)?;
            Ok(Candidate::new(Source::Flathub, name, name))
        }
        Source::Nix => {
            nix::validate_attr(name)?;
            if ctx.check_nix && !(ctx.nix_exists)(name)? {
                return Err(ResolveError::UnknownAttr(name.to_string()));
            }
            Ok(Candidate::new(Source::Nix, name, name))
        }
    }
}

fn source_str<S: Serializer>(source: &Source, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(source.as_str())
}

fn sources_str<S: Serializer>(sources: &[Source], s: S) -> Result<S::Ok, S::Error> {
    s.collect_seq(sources.iter().map(|source| source.as_str()))
}
