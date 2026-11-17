use std::time::SystemTime;

use anyhow::Result;
use kit_core::catalog::{catalog_override, Loader};
use kit_core::facile::{self_dir, Backend, Finding, Owner, KIT_REPO};
use serde_json::json;

use super::doctor_more::{backends, sources, Report};
use super::{Ctx, VERSION};
use crate::ui::Failure;

/// `kit doctor`: report anything that would make kit or a Facile tool misbehave here.
pub fn run(ctx: &Ctx) -> Result<()> {
    let mut report = Report::new(ctx);
    ctx.ui
        .step(&format!("Install directory {}", ctx.tilde(&ctx.bin_dir)));
    let backend = ctx.backend()?;
    tools(ctx, &backend, &mut report);
    catalog(&mut report)?;
    backends(ctx, &mut report);
    sources(ctx, &mut report)?;
    if report.found.is_empty() {
        ctx.ui.success("Everything looks healthy");
    }
    report_self(ctx, &backend);
    let problems = report.found.len();
    if ctx.ui.json() {
        ctx.ui
            .emit(&json!({ "version": VERSION, "problems": problems, "findings": report.found }));
    }
    match problems {
        0 => Ok(()),
        _ if ctx.ui.json() => Err(Failure::silent(1).into()),
        n => Err(Failure::new(format!("{n} problem(s) found"), Vec::new()).into()),
    }
}

fn tools(ctx: &Ctx, backend: &Backend, report: &mut Report) {
    let exe = std::env::current_exe().unwrap_or_default();
    match ctx.catalog() {
        Ok(manifest) => {
            for tool in backend.installed(&manifest, &ctx.bin_dir) {
                ctx.ui.success(&tool.line);
            }
            let found = backend.doctor(&manifest, &ctx.bin_dir, &exe);
            found.into_iter().for_each(|f| report.problem(f));
        }
        Err(e) => report.problem(finding(format!("{e:#}"), Vec::new())),
    }
}

/// A finding with its hints.
pub fn finding(message: String, hints: Vec<String>) -> Finding {
    Finding { message, hints }
}

fn catalog(report: &mut Report) -> Result<()> {
    let local = catalog_override(
        std::env::var_os("KIT_CATALOG"),
        std::env::var_os("FACILE_CATALOG"),
    );
    if local.is_some() {
        return Ok(());
    }
    if Loader::from_env()?.refresh().is_err() {
        report.problem(finding(
            "cannot reach the tool catalog, working from the cached or built-in copy".into(),
            Vec::new(),
        ));
    }
    Ok(())
}

fn report_self(ctx: &Ctx, backend: &Backend) {
    let repos = [KIT_REPO.to_string()];
    let latest = backend
        .latest_cache()
        .resolve(&backend.github, &repos, SystemTime::now());
    let tag = latest
        .get(KIT_REPO)
        .map(|t| t.trim_start_matches('v'))
        .unwrap_or_default();
    if !kit_core::facile::outdated(VERSION, tag) {
        ctx.ui.success(&format!("kit {VERSION}"));
        return;
    }
    let owner = std::env::current_exe()
        .ok()
        .and_then(|exe| self_dir(&exe).err())
        .unwrap_or(Owner::Kit);
    ctx.ui.warn(&format!("kit {VERSION} is behind {tag}"));
    ctx.ui.hint(&owner.update_command("kit"));
}
