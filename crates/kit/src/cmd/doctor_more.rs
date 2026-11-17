use std::time::Duration;

use anyhow::Result;
use kit_core::catalog::{sources_path, Fetch, HttpFetch, ListSource, Loader};
use kit_core::facile::Finding;
use kit_core::listfile::Source;

use super::backends::{self as installed, hint_for};
use super::doctor::finding;
use super::Ctx;

/// The problems doctor found so far. Each one is printed when it is found.
pub struct Report<'a> {
    ctx: &'a Ctx<'a>,
    /// Every problem, in the order found.
    pub found: Vec<Finding>,
}

impl<'a> Report<'a> {
    /// An empty report printing through `ctx`.
    pub fn new(ctx: &'a Ctx<'a>) -> Report<'a> {
        Report {
            ctx,
            found: Vec::new(),
        }
    }

    /// Prints a problem as a warning with its hints and records it.
    pub fn problem(&mut self, f: Finding) {
        self.ctx.ui.warn(&f.message);
        f.hints.iter().for_each(|h| self.ctx.ui.hint(h));
        self.found.push(f);
    }
}

/// The list file parses, and every backend it names is on this machine.
pub fn backends(ctx: &Ctx, report: &mut Report) {
    let file = match ctx.list() {
        Ok(file) => file,
        Err(e) => return report.problem(finding(format!("{e:#}"), Vec::new())),
    };
    let count: usize = Source::ALL.iter().map(|s| file.entries(*s).len()).sum();
    ctx.ui.success(&format!(
        "List file {} ({count} package(s))",
        ctx.tilde(&ctx.file)
    ));
    if !file.flathub.is_empty() && installed::flatpak().is_none() {
        let message = format!(
            "{} Flathub app(s) listed but flatpak is not installed",
            file.flathub.len()
        );
        report.problem(finding(message, vec![hint_for(Source::Flathub).into()]));
    }
    if !file.nix.is_empty() && installed::nix().ok().flatten().is_none() {
        let message = format!(
            "{} Nix package(s) listed but nix is not installed",
            file.nix.len()
        );
        report.problem(finding(message, vec![hint_for(Source::Nix).into()]));
    }
}

/// The user's sources file parses, its list URLs answer, and no entry shadows a catalog tool.
pub fn sources(ctx: &Ctx, report: &mut Report) -> Result<()> {
    let loader = Loader::from_env()?;
    let Some(path) = sources_path(&loader.sources) else {
        return Ok(());
    };
    ctx.ui.step(&format!("Sources file {}", ctx.tilde(&path)));
    let config = match loader.load_sources() {
        Ok(config) => config,
        Err(e) => {
            report.problem(finding(e.to_string(), Vec::new()));
            return Ok(());
        }
    };
    ctx.ui.success("Sources file healthy");
    lists(ctx, report, &config.lists);
    if let Ok(base) = loader.load_base() {
        for name in base.shadowed_by(&config) {
            let message = format!("user source {name:?} shadows the catalog tool of the same name");
            report.problem(finding(message, Vec::new()));
        }
    }
    if let Ok(merged) = loader.load() {
        merged
            .load_errors
            .into_iter()
            .for_each(|e| report.problem(finding(e, Vec::new())));
    }
    Ok(())
}

fn lists(ctx: &Ctx, report: &mut Report, lists: &[ListSource]) {
    let fetch = HttpFetch::new(Duration::from_secs(10));
    for list in lists {
        match fetch.get(&list.url) {
            Ok(_) => ctx
                .ui
                .success(&format!("list source {:?} reachable", list.name)),
            Err(e) => report.problem(finding(
                format!("list source {:?} unreachable: {e}", list.name),
                Vec::new(),
            )),
        }
    }
}
