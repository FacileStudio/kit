use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::runlog::{build_command, run};
use super::FacileError;
use crate::catalog::Tool;

/// Where one source build happens.
pub(crate) struct Checkout<'a> {
    pub(crate) src: &'a Path,
    pub(crate) root: &'a Path,
    pub(crate) out: &'a Path,
}

/// Builds with the toolchain the tool declares and returns one path per binary, in tool order.
pub(crate) fn build_for(tool: &Tool, at: &Checkout) -> Result<Vec<PathBuf>, FacileError> {
    match tool.build.as_str() {
        "rust" => build_rust(tool, at),
        "go" => build_go(tool, at),
        _ => build_bun(tool, at),
    }
}

fn build_rust(tool: &Tool, at: &Checkout) -> Result<Vec<PathBuf>, FacileError> {
    let target = at.src.join("target");
    let mut cmd = build_command("cargo", &["build", "--release", "--quiet"], at.src, at.root);
    run(cmd.env("CARGO_TARGET_DIR", &target))?;
    collect(tool, &target.join("release"), at.out, "cargo")
}

fn build_go(tool: &Tool, at: &Checkout) -> Result<Vec<PathBuf>, FacileError> {
    let bins = tool.bin_list();
    if bins.len() > 1 {
        return Err(FacileError::Build(format!(
            "the go backend builds one binary from {}: split the tool or publish a release asset",
            tool.src_subdir
        )));
    }
    let mut ldflags = "-s -w".to_string();
    if !tool.go_version_var.is_empty() {
        ldflags.push_str(&format!(
            " -X {}={}",
            tool.go_version_var,
            describe(at.root)
        ));
    }
    let dest = at.out.join(&bins[0]);
    let dest_arg = dest.to_string_lossy().to_string();
    let args = [
        "build",
        "-trimpath",
        "-ldflags",
        &ldflags,
        "-o",
        &dest_arg,
        ".",
    ];
    run(&mut build_command("go", &args, at.src, at.root))?;
    Ok(vec![dest])
}

fn build_bun(tool: &Tool, at: &Checkout) -> Result<Vec<PathBuf>, FacileError> {
    let install = ["install", "--frozen-lockfile", "--silent"];
    run(&mut build_command("bun", &install, at.src, at.root))?;
    run(&mut build_command(
        "bun",
        &["run", "--silent", "build"],
        at.src,
        at.root,
    ))?;
    collect(tool, at.src, at.out, "bun")
}

fn collect(
    tool: &Tool,
    from: &Path,
    out: &Path,
    toolchain: &str,
) -> Result<Vec<PathBuf>, FacileError> {
    tool.bin_list()
        .iter()
        .map(|bin| {
            let dest = out.join(bin);
            fs::rename(from.join(bin), &dest)
                .map(|()| dest)
                .map_err(|e| {
                    FacileError::Build(format!("{toolchain} finished but {bin} is missing: {e}"))
                })
        })
        .collect()
}

/// The checkout named as a tag, or `dev` when git cannot describe it.
fn describe(checkout: &Path) -> String {
    let out = Command::new("git")
        .args(["describe", "--tags", "--always"])
        .current_dir(checkout)
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            text.trim().trim_start_matches('v').to_string()
        }
        _ => "dev".to_string(),
    }
}
