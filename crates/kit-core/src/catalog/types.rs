use serde::{Deserialize, Serialize};

/// One installable CLI. Field names mirror the config block of each repo's install.sh.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Tool {
    /// The name users type.
    pub name: String,
    /// One line on what the tool does.
    pub summary: String,
    /// The GitHub `owner/repo`.
    pub repo: String,
    /// The branch a source build checks out.
    pub branch: String,
    /// The primary binary, the one that answers `--version`.
    pub bin: String,
    /// Every binary a workspace repo installs, primary first.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bins: Vec<String>,
    /// The toolchain: go or rust.
    pub build: String,
    /// The directory inside the repo the build runs in.
    pub src_subdir: String,
    /// The release asset name prefix.
    pub asset: String,
    /// The agent skill the tool ships, if any.
    pub skill: String,
    /// The Go `-X` variable that receives the version.
    pub go_version_var: String,
    /// Commands that must be on PATH for the tool to work.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    /// The command that prints the version, when `--version` does not.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub version_cmd: String,
    /// The regex that extracts the version from that output.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub version_pattern: String,
    /// The login description, kept as opaque data: kit does not log in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<serde_yaml::Value>,
}

impl Tool {
    /// Every binary the tool installs, in install order.
    pub fn bin_list(&self) -> Vec<String> {
        if self.bins.is_empty() {
            vec![self.bin.clone()]
        } else {
            self.bins.clone()
        }
    }
}

/// The whole catalog.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Manifest {
    /// The catalog format version.
    pub version: i64,
    /// Every tool, in catalog order.
    pub tools: Vec<Tool>,
    /// Tools whose `install` subcommand registers an MCP server after kit places them.
    pub mcp: Vec<String>,
    /// Non-fatal problems met while merging user sources.
    #[serde(skip)]
    pub load_errors: Vec<String>,
}

impl Manifest {
    /// Parses a catalog. A catalog with no tools is an error; unknown keys are ignored.
    pub fn parse(raw: &str) -> Result<Manifest, super::CatalogError> {
        let m: Manifest = serde_yaml::from_str(raw)
            .map_err(|e| super::CatalogError::Invalid(format!("parsing catalog: {e}")))?;
        if m.tools.is_empty() {
            return Err(super::CatalogError::NoTools);
        }
        Ok(m)
    }

    /// Parses catalog bytes, which must be UTF-8.
    pub fn parse_bytes(raw: &[u8]) -> Result<Manifest, super::CatalogError> {
        let text = std::str::from_utf8(raw)
            .map_err(|e| super::CatalogError::Invalid(format!("catalog is not UTF-8: {e}")))?;
        Manifest::parse(text)
    }

    /// The tool with this name, compared case-insensitively.
    pub fn get(&self, name: &str) -> Option<&Tool> {
        self.tools
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(name))
    }

    /// Every tool name in catalog order.
    pub fn names(&self) -> Vec<String> {
        self.tools.iter().map(|t| t.name.clone()).collect()
    }

    /// Names of user single sources that replace a catalog tool of the same name.
    pub fn shadowed_by(&self, sources: &super::SourceConfig) -> Vec<String> {
        sources
            .single
            .iter()
            .filter(|s| !s.name.is_empty() && self.get(&s.name).is_some())
            .map(|s| s.name.clone())
            .collect()
    }
}
