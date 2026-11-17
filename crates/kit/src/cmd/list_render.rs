use kit_core::facile::{is_semver, self_dir};

use super::list::{Listing, Row};
use super::Ctx;
use crate::ui::style::Tone;

/// What the state column says for a row.
pub fn state_of(row: &Row) -> String {
    match row {
        r if r.outdated => format!("{} → {}", r.version, r.latest),
        r if r.installed && !r.version.is_empty() => r.version.clone(),
        r if r.installed => "installed".into(),
        r if !r.note.is_empty() => r.note.clone(),
        _ => "not installed".into(),
    }
}

fn tone_of(row: &Row) -> Tone {
    match row {
        r if r.outdated => Tone::Accent,
        r if !r.installed => Tone::Dim,
        r if r.version.contains("dev") => Tone::Notice,
        r if !r.version.is_empty() && !is_semver(&r.version) => Tone::Alert,
        _ => Tone::Good,
    }
}

/// Prints the listing as aligned sections. Padding is done before colouring: escapes have no width.
pub fn render(ctx: &Ctx, listing: &Listing) {
    let all = || {
        listing
            .facile
            .iter()
            .chain(&listing.nix)
            .chain(&listing.flathub)
    };
    let widths = (
        all().map(|r| r.name.len()).max().unwrap_or(0),
        all()
            .map(|r| state_of(r).chars().count())
            .max()
            .unwrap_or(0),
    );
    section(ctx, "facile", &listing.facile, widths);
    section(ctx, "nix", &listing.nix, widths);
    section(ctx, "flathub", &listing.flathub, widths);
    if listing.facile.len() == 1 && listing.nix.is_empty() && listing.flathub.is_empty() {
        ctx.ui
            .out(&ctx.ui.dim("No packages listed, run `kit add <name>`"));
    }
    footer(ctx, listing);
}

fn section(ctx: &Ctx, title: &str, rows: &[Row], (name_w, state_w): (usize, usize)) {
    if rows.is_empty() {
        return;
    }
    ctx.ui.out(&ctx.ui.tint(Tone::Accent, title));
    for row in rows {
        let state = ctx
            .ui
            .tint(tone_of(row), &format!("{:<state_w$}", state_of(row)));
        let line = format!(
            "  {:<name_w$}  {state}  {}",
            row.name,
            ctx.ui.dim(&row.summary)
        );
        ctx.ui.out(line.trim_end());
    }
}

fn footer(ctx: &Ctx, listing: &Listing) {
    let managed = std::env::current_exe()
        .ok()
        .and_then(|exe| self_dir(&exe).err());
    let count = listing
        .facile
        .iter()
        .filter(|r| r.outdated && !(r.name == "kit" && managed.is_some()))
        .count();
    match count {
        0 => {}
        1 => ctx
            .ui
            .out(&ctx.ui.dim("1 update available, run `kit update`")),
        n => ctx.ui.out(
            &ctx.ui
                .dim(&format!("{n} updates available, run `kit update`")),
        ),
    }
    if let Some(owner) = managed.filter(|_| listing.facile.first().is_some_and(|r| r.outdated)) {
        let line = format!(
            "kit is managed by {owner}, run `{}`",
            owner.update_command("kit")
        );
        ctx.ui.out(&ctx.ui.dim(&line));
    }
}
