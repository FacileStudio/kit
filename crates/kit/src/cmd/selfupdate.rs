use anyhow::{Context as _, Result};
use kit_core::facile::{SelfOptions, SelfOutcome};
use serde_json::{json, Value};

use super::{Ctx, VERSION};
use crate::cli::SelfArgs;
use crate::ui::Failure;

/// `kit self-update`.
pub fn run(ctx: &Ctx, args: SelfArgs) -> Result<()> {
    let outcome = update_self(ctx, args)?;
    if ctx.ui.json() {
        ctx.ui.emit(&outcome);
    }
    Ok(())
}

/// Replaces the running kit at its own path, or says which package manager owns it.
pub fn update_self(ctx: &Ctx, args: SelfArgs) -> Result<Value> {
    let backend = ctx.backend()?;
    let exe = std::env::current_exe().context("cannot locate the running kit")?;
    let opts = SelfOptions {
        force: args.force,
        source: args.source,
    };
    let outcome = backend
        .self_update(VERSION, &exe, opts, ctx.ui)
        .map_err(|e| Failure::new(format!("kit did not update: {e}"), Vec::new()))?;
    Ok(describe(ctx, outcome))
}

fn describe(ctx: &Ctx, outcome: SelfOutcome) -> Value {
    match outcome {
        SelfOutcome::Updated(line, path) => {
            ctx.ui
                .success(&format!("{line} installed to {}", ctx.tilde(&path)));
            json!({ "outcome": "updated", "version": line })
        }
        SelfOutcome::UpToDate(v) => {
            ctx.ui.success(&format!("kit {v} is up to date"));
            json!({ "outcome": "current", "version": v })
        }
        SelfOutcome::SourceBuild(v) => {
            ctx.ui
                .step(&format!("kit {v} is a source build, leaving it alone"));
            ctx.ui
                .hint("kit self-update --force replaces it with the published release");
            json!({ "outcome": "source-build", "version": v })
        }
        SelfOutcome::Managed {
            owner,
            behind,
            command,
        } => {
            managed(ctx, &owner.to_string(), behind, &command);
            json!({ "outcome": "managed", "owner": owner, "behind": behind, "command": command })
        }
    }
}

fn managed(ctx: &Ctx, owner: &str, behind: bool, command: &str) {
    if !behind {
        ctx.ui.success(&format!("kit {VERSION} is up to date"));
        return;
    }
    ctx.ui.step(&format!("kit {VERSION} is managed by {owner}"));
    ctx.ui.hint(command);
}
