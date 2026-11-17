//! The bare-name lookup chain: Facile catalog, then Flathub, then Nix.

use super::{Candidate, Context, Outcome, Resolution, ResolveError};
use crate::flathub::FlathubError;
use crate::listfile::Source;
use crate::nix::{self, NixError};

/// Tries each backend in order and stops at the first that matches.
pub(super) fn resolve(name: &str, ctx: &Context<'_>) -> Result<Outcome, ResolveError> {
    let mut skipped = Vec::new();
    if let Some(tool) = (ctx.catalog)(name)? {
        let one = Candidate::new(Source::Facile, &tool.name, &tool.name);
        return Ok(found(vec![one], skipped));
    }
    let mut matches = flathub(name, ctx, &mut skipped)?;
    if matches.is_empty() {
        matches = nix(name, ctx, &mut skipped)?;
    }
    Ok(found(matches, skipped))
}

fn flathub(
    name: &str,
    ctx: &Context<'_>,
    skipped: &mut Vec<Source>,
) -> Result<Vec<Candidate>, ResolveError> {
    match (ctx.flathub_search)(name) {
        Ok(hits) => Ok(flathub_matches(name, hits)),
        Err(FlathubError::NotAvailable) => {
            skipped.push(Source::Flathub);
            Ok(Vec::new())
        }
        Err(e) => Err(e.into()),
    }
}

fn nix(
    name: &str,
    ctx: &Context<'_>,
    skipped: &mut Vec<Source>,
) -> Result<Vec<Candidate>, ResolveError> {
    if nix::validate_attr(name).is_err() {
        return Ok(Vec::new());
    }
    match (ctx.nix_exists)(name) {
        Ok(true) => Ok(vec![Candidate::new(Source::Nix, name, name)]),
        Ok(false) => Ok(Vec::new()),
        Err(NixError::NotAvailable(_)) => {
            skipped.push(Source::Nix);
            Ok(Vec::new())
        }
        Err(e) => Err(e.into()),
    }
}

fn found(mut candidates: Vec<Candidate>, skipped: Vec<Source>) -> Outcome {
    let resolution = match candidates.len() {
        0 => Resolution::None,
        1 => Resolution::One(candidates.remove(0)),
        _ => Resolution::Many(candidates),
    };
    Outcome {
        resolution,
        skipped,
    }
}

fn flathub_matches(name: &str, hits: Vec<(String, String)>) -> Vec<Candidate> {
    let mut out: Vec<Candidate> = Vec::new();
    for (id, display) in hits {
        if is_match(name, &id, &display) && !out.iter().any(|c| c.id == id) {
            out.push(Candidate::new(Source::Flathub, &id, &display));
        }
    }
    match out.iter().position(|c| c.id == name) {
        Some(exact) => vec![out.remove(exact)],
        None => out,
    }
}

fn is_match(name: &str, id: &str, display: &str) -> bool {
    let last = id.rsplit('.').next().unwrap_or(id);
    id == name || display.eq_ignore_ascii_case(name) || last.eq_ignore_ascii_case(name)
}
