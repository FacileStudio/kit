use serde::Deserialize;

use super::{spec::valid_pattern, CatalogError, SingleSource};

#[derive(Deserialize)]
struct MiseToml {
    facile: Option<SingleSource>,
}

#[derive(Deserialize)]
struct NameOnly {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct MiseProbe {
    facile: Option<NameOnly>,
}

fn decode<T: for<'de> Deserialize<'de>>(raw: &[u8], file: &str) -> Result<T, CatalogError> {
    let text = std::str::from_utf8(raw)
        .map_err(|e| CatalogError::Invalid(format!("parsing {file}: {e}")))?;
    toml::from_str(text).map_err(|e| CatalogError::Invalid(format!("parsing {file}: {e}")))
}

fn validate(spec: SingleSource) -> Result<SingleSource, CatalogError> {
    let fail = |what: String| Err(CatalogError::Invalid(format!("facile.toml: {what}")));
    if spec.name.is_empty() {
        return fail("name is required".into());
    }
    if spec.repo.is_empty() {
        return fail("repo is required".into());
    }
    if !valid_pattern(&spec.version_pattern) {
        return fail(format!("invalid versionPattern: {}", spec.version_pattern));
    }
    Ok(spec)
}

/// Parses a repo's facile.toml. Name and repo are required and the version pattern must compile.
pub(crate) fn parse_facile_toml(raw: &[u8]) -> Result<SingleSource, CatalogError> {
    validate(decode(raw, "facile.toml")?)
}

/// Parses the `[facile]` block of a mise.toml, with the same rules as facile.toml.
pub(crate) fn parse_mise_toml(raw: &[u8]) -> Result<SingleSource, CatalogError> {
    let doc: MiseToml = decode(raw, "mise.toml")?;
    validate(doc.facile.ok_or(CatalogError::NoFacileToml)?)
}

/// True when the bytes are TOML with a non-empty top-level `name`.
pub(crate) fn facile_toml_probe(raw: &[u8]) -> bool {
    decode::<NameOnly>(raw, "facile.toml").is_ok_and(|d| !d.name.is_empty())
}

/// True when the bytes are TOML with a `[facile]` block that has a name.
pub(crate) fn mise_toml_probe(raw: &[u8]) -> bool {
    decode::<MiseProbe>(raw, "mise.toml")
        .is_ok_and(|d| d.facile.is_some_and(|f| !f.name.is_empty()))
}
