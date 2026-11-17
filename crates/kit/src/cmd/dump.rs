use anyhow::Result;
use kit_core::listfile::Source;
use serde_json::json;

use super::check::measure;
use super::drift::Item;
use super::Ctx;

/// `kit dump`: add installed Facile tools and Flathub apps that are not listed. Nix is already the file.
pub fn run(ctx: &Ctx, yes: bool) -> Result<()> {
    let extra: Vec<Item> = measure(ctx, false)?
        .extra
        .into_iter()
        .filter(|i| i.source != Source::Nix.as_str())
        .collect();
    if extra.is_empty() {
        return finish(ctx, &extra, "Everything installed is already listed");
    }
    extra
        .iter()
        .for_each(|i| ctx.ui.out(&format!("+ {}:{}", i.source, i.id)));
    let question = format!(
        "Add {} package(s) to {}?",
        extra.len(),
        ctx.tilde(&ctx.file)
    );
    if !ctx.ui.confirm(&question, yes)? {
        ctx.ui.step("Nothing changed");
        return Ok(());
    }
    record(ctx, &extra)?;
    let file = ctx.tilde(&ctx.file);
    finish(
        ctx,
        &extra,
        &format!("Added {} package(s) to {file}", extra.len()),
    )
}

fn record(ctx: &Ctx, extra: &[Item]) -> Result<()> {
    ctx.edit(|file| {
        for item in extra {
            if let Ok(source) = item.source.parse::<Source>() {
                file.add(source, &item.id);
            }
        }
    })
}

fn finish(ctx: &Ctx, added: &[Item], message: &str) -> Result<()> {
    if ctx.ui.json() {
        ctx.ui.emit(&json!({ "file": ctx.file, "added": added }));
    }
    ctx.ui.success(message);
    Ok(())
}
