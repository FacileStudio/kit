use super::state::ENV_RECORD;
use super::NixError;

const PKGS: &str =
    "let pkgs = (builtins.getFlake \"nixpkgs\").legacyPackages.${builtins.currentSystem}; in";
const MAX_LEN: usize = 200;

/// Checks that `attr` is a dotted attribute path safe to splice into a Nix expression.
///
/// Each dot-separated name starts with a letter or `_` and holds only letters, digits, `-` and `_`.
pub fn validate_attr(attr: &str) -> Result<(), NixError> {
    let name_ok = |name: &str| {
        let mut chars = name.chars();
        chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    };
    if attr.len() <= MAX_LEN && attr.split('.').all(name_ok) {
        return Ok(());
    }
    Err(NixError::InvalidAttr(attr.to_string()))
}

/// Validates every attribute, then sorts and deduplicates the list.
pub fn normalize(attrs: &[String]) -> Result<Vec<String>, NixError> {
    attrs.iter().try_for_each(|a| validate_attr(a))?;
    let mut out = attrs.to_vec();
    out.sort();
    out.dedup();
    Ok(out)
}

/// Reads the locked revision from `nix flake metadata --json` output.
pub fn parse_rev(json: &str) -> Result<String, NixError> {
    let meta: serde_json::Value = serde_json::from_str(json)?;
    let rev = meta
        .pointer("/locked/rev")
        .or_else(|| meta.get("revision"))
        .and_then(|v| v.as_str());
    match rev {
        Some(r)
            if !r.is_empty() && r.len() <= 64 && r.chars().all(|c| c.is_ascii_alphanumeric()) =>
        {
            Ok(r.to_string())
        }
        _ => Err(NixError::BadOutput(
            "nix flake metadata nixpkgs has no locked revision".to_string(),
        )),
    }
}

/// The `buildEnv` expression for `attrs`, with a record of `rev` and `attrs` inside the environment.
pub fn env_expr(rev: &str, attrs: &[String]) -> Result<String, NixError> {
    attrs.iter().try_for_each(|a| validate_attr(a))?;
    if !rev.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(NixError::BadOutput(format!("bad revision {rev:?}")));
    }
    let paths: String = attrs.iter().map(|a| format!("pkgs.{a} ")).collect();
    let names: String = attrs.iter().map(|a| format!("\"{a}\" ")).collect();
    let record = format!("builtins.toJSON {{ rev = \"{rev}\"; attrs = [ {names}]; }}");
    Ok(format!(
        "{PKGS} pkgs.buildEnv {{ name = \"kit-env\"; paths = [ {paths}(pkgs.writeTextDir \"{ENV_RECORD}\" ({record})) ]; }}"
    ))
}

/// The expression that evaluates to `true` when `attr` exists in the registry nixpkgs.
pub fn exists_expr(attr: &str) -> Result<String, NixError> {
    validate_attr(attr)?;
    Ok(format!("{PKGS} pkgs ? {attr}"))
}
