use anyhow::Result;
use kit_core::listfile::Source;
use kit_core::nix::State;
use serde_json::json;

use super::backends::{self, NIX_HINT};
use super::Ctx;
use crate::ui::Failure;

/// `kit undo`: roll kit's Nix profile back one generation and make the list file say what it now holds.
pub fn run(ctx: &Ctx, source: &str) -> Result<()> {
    let source: Source = source.parse()?;
    if source != Source::Nix {
        return Err(Failure::new(
            format!("undo is not available for {source}: only nix keeps generations"),
            vec![format!(
                "run `kit add` or `kit del` to change {source} packages"
            )],
        )
        .into());
    }
    let Some(nix) = backends::nix()? else {
        return Err(Failure::new("nix is not installed", vec![NIX_HINT.into()]).into());
    };
    let restored = nix.undo()?;
    record(ctx, &restored)?;
    if ctx.ui.json() {
        ctx.ui.emit(
            &json!({ "rev": restored.rev, "attrs": restored.attrs, "stale": restored.stale }),
        );
    }
    Ok(())
}

fn record(ctx: &Ctx, restored: &State) -> Result<()> {
    if restored.stale {
        ctx.ui
            .warn("kit cannot tell what the restored generation holds, the list file is unchanged");
        ctx.ui
            .hint("run `kit sync` to rebuild the profile from the list file");
        return Ok(());
    }
    ctx.edit(|file| file.nix = restored.attrs.clone())?;
    let count = restored.attrs.len();
    ctx.ui.success(&format!(
        "Nix profile rolled back to {count} package(s), list file updated"
    ));
    Ok(())
}
