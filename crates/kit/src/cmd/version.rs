use anyhow::Result;
use serde_json::json;

use super::{Ctx, VERSION};

/// `kit version`: the same line as `kit --version`.
pub fn run(ctx: &Ctx) -> Result<()> {
    if ctx.ui.json() {
        ctx.ui.emit(&json!({ "version": VERSION }));
    } else {
        println!("kit {VERSION}");
    }
    Ok(())
}
