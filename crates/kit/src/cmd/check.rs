use anyhow::Result;
use kit_core::listfile::Source;

use super::backends::hint_for;
use super::drift::{compute, Drift, Item};
use super::machine::{self, probe};
use super::Ctx;
use crate::ui::style::Tone;
use crate::ui::Failure;

/// `kit check`: report drift between the list file and the machine. Exits 1 when there is any.
pub fn run(ctx: &Ctx) -> Result<()> {
    let drift = measure(ctx, true)?;
    if ctx.ui.json() {
        ctx.ui.emit(&drift);
    } else {
        render(ctx, &drift);
    }
    if drift.is_clean() {
        return Ok(());
    }
    Err(Failure::silent(1).into())
}

/// The drift right now. With `with_latest`, outdated Facile tools are found through the daily tag cache.
pub fn measure(ctx: &Ctx, with_latest: bool) -> Result<Drift> {
    let file = ctx.list()?;
    let manifest = ctx.catalog()?;
    let backend = ctx.backend()?;
    let machine = probe(ctx, &backend, &manifest)?;
    let latest = match with_latest {
        true => machine::latest(&backend, &manifest, &machine, false),
        false => Default::default(),
    };
    Ok(compute(&file, &machine, &manifest, &latest))
}

fn render(ctx: &Ctx, drift: &Drift) {
    for name in &drift.skipped {
        let source: Source = name.parse().unwrap_or(Source::Nix);
        ctx.ui
            .warn(&format!("{name} is not available on this machine"));
        ctx.ui.hint(hint_for(source));
    }
    section(ctx, &drift.missing, ("+", Tone::Good, "not installed"));
    section(
        ctx,
        &drift.extra,
        ("-", Tone::Alert, "installed, not listed"),
    );
    section(ctx, &drift.outdated, ("~", Tone::Accent, ""));
    if drift.is_clean() {
        ctx.ui
            .success(&format!("This machine matches {}", ctx.tilde(&ctx.file)));
        return;
    }
    let mut hints = Vec::new();
    if !drift.missing.is_empty() {
        hints.push("run `kit sync` to install what is missing");
    }
    if !drift.extra.is_empty() {
        hints.push("run `kit dump` to list what is extra, or `kit clean` to remove it");
    }
    if !drift.outdated.is_empty() {
        hints.push("run `kit update` to update what is outdated");
    }
    hints.iter().for_each(|h| ctx.ui.out(&ctx.ui.dim(h)));
}

fn section(ctx: &Ctx, items: &[Item], (glyph, tone, note): (&str, Tone, &str)) {
    for item in items {
        let detail = if item.latest.is_empty() {
            note.to_string()
        } else {
            format!("{} → {}", item.version, item.latest)
        };
        let id = format!("{}:{}", item.source, item.id);
        ctx.ui.out(&format!(
            "{} {id}  {}",
            ctx.ui.tint(tone, glyph),
            ctx.ui.dim(&detail)
        ));
    }
}
