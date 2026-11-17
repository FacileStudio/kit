use std::process::{Command, Stdio};

use anyhow::{Context as _, Result};
use kit_core::paths;
use serde_json::json;

use super::backends;
use super::Ctx;
use crate::cli::SetupTarget;
use crate::ui::Failure;

/// The upstream multi-user Nix installer, run exactly as nixos.org documents it.
pub const NIX_INSTALL: &str = "curl -sSfL https://nixos.org/nix/install | sh -s -- --daemon";

/// `kit setup <backend>`.
pub fn run(ctx: &Ctx, target: &SetupTarget) -> Result<()> {
    match target {
        SetupTarget::Nix(args) => nix(ctx, args.yes),
    }
}

fn nix(ctx: &Ctx, yes: bool) -> Result<()> {
    if backends::nix()?.is_some() {
        ctx.ui.success("Nix is already installed");
        return advise(ctx, false);
    }
    ctx.ui
        .step("kit will run the upstream Nix installer in multi-user mode:");
    ctx.ui.hint(NIX_INSTALL);
    ctx.ui.hint(
        "the installer asks for sudo itself; kit never does and never edits your shell files",
    );
    if !ctx.ui.confirm("Run it now?", yes)? {
        ctx.ui.step("Nothing changed");
        return Ok(());
    }
    run_installer(ctx)?;
    ctx.ui.success("Nix installed");
    advise(ctx, true)
}

fn run_installer(ctx: &Ctx) -> Result<()> {
    let stdout = if ctx.ui.json() {
        Stdio::from(std::io::stderr())
    } else {
        Stdio::inherit()
    };
    let status = Command::new("sh")
        .args(["-c", NIX_INSTALL])
        .stdout(stdout)
        .status()
        .context("cannot start sh")?;
    if status.success() {
        return Ok(());
    }
    let failure = Failure::new(
        "the Nix installer failed",
        vec!["read its output above, then retry".into()],
    );
    Err(Failure {
        code: status.code().unwrap_or(1),
        ..failure
    }
    .into())
}

fn advise(ctx: &Ctx, installed: bool) -> Result<()> {
    let profile = paths::state_dir()?.join("nix-profile").join("bin");
    let export = format!("export PATH=\"{}:$PATH\"", profile.display());
    if ctx.ui.json() {
        ctx.ui
            .emit(&json!({ "installed": installed, "path": export }));
        return Ok(());
    }
    ctx.ui.out("Open a new shell, or load Nix in this one:");
    ctx.ui
        .out("  . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh");
    ctx.ui
        .out("Put kit's Nix packages on your PATH in your shell profile:");
    ctx.ui.out(&format!("  {export}"));
    Ok(())
}
