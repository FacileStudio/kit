use serde::{Deserialize, Serialize};

use super::{CatalogError, Tool};

/// A tool described by the user or by a repo's facile.toml. Missing fields get defaults in `to_tool`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SingleSource {
    /// The name users type.
    pub name: String,
    /// One line on what the tool does.
    pub summary: String,
    /// The GitHub `owner/repo`.
    pub repo: String,
    /// The branch, `main` when empty.
    pub branch: String,
    /// The primary binary, the name when empty.
    pub bin: String,
    /// Every binary, primary first.
    pub bins: Vec<String>,
    /// The toolchain, `go` when empty.
    pub build: String,
    /// The build directory, `.` when empty.
    pub src_subdir: String,
    /// The release asset name prefix.
    pub asset: String,
    /// The agent skill the tool ships.
    pub skill: String,
    /// The Go `-X` variable that receives the version.
    pub go_version_var: String,
    /// Commands that must be on PATH.
    pub requires: Vec<String>,
    /// The command that prints the version.
    pub version_cmd: String,
    /// The regex that extracts the version.
    pub version_pattern: String,
}

fn or(value: &str, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

impl SingleSource {
    /// The tool this entry describes, with defaults filled. A missing name or a bad pattern is an error.
    pub fn to_tool(&self) -> Result<Tool, CatalogError> {
        if self.name.is_empty() {
            return Err(CatalogError::Invalid(
                "single source entry missing name".into(),
            ));
        }
        if !valid_pattern(&self.version_pattern) {
            return Err(CatalogError::Invalid(format!(
                "single source {:?}: invalid versionPattern: {}",
                self.name, self.version_pattern
            )));
        }
        Ok(self.defaults())
    }

    pub(crate) fn defaults(&self) -> Tool {
        let mut bin = or(&self.bin, &self.name);
        let bins = match self.bins.first() {
            Some(first) => {
                bin = first.clone();
                self.bins.clone()
            }
            None => vec![bin.clone()],
        };
        Tool {
            name: self.name.clone(),
            summary: self.summary.clone(),
            repo: self.repo.clone(),
            branch: or(&self.branch, "main"),
            bin,
            bins,
            build: or(&self.build, "go"),
            src_subdir: or(&self.src_subdir, "."),
            asset: self.asset.clone(),
            skill: self.skill.clone(),
            go_version_var: self.go_version_var.clone(),
            requires: self.requires.clone(),
            version_cmd: self.version_cmd.clone(),
            version_pattern: self.version_pattern.clone(),
            auth: None,
        }
    }

    pub(crate) fn apply_to(&self, t: &mut Tool) {
        let fields = [
            (&self.name, &mut t.name),
            (&self.branch, &mut t.branch),
            (&self.build, &mut t.build),
            (&self.asset, &mut t.asset),
            (&self.summary, &mut t.summary),
            (&self.version_cmd, &mut t.version_cmd),
            (&self.version_pattern, &mut t.version_pattern),
        ];
        for (from, to) in fields {
            if !from.is_empty() {
                *to = from.clone();
            }
        }
        if !self.bin.is_empty() {
            t.bin = self.bin.clone();
            if t.bins.len() <= 1 {
                t.bins = vec![self.bin.clone()];
            }
        }
        if let Some(first) = self.bins.first() {
            t.bin = first.clone();
            t.bins = self.bins.clone();
        }
    }
}

/// True when the pattern is empty or compiles. A bad pattern would make every version look current.
pub(crate) fn valid_pattern(pattern: &str) -> bool {
    pattern.is_empty() || regex::Regex::new(pattern).is_ok()
}

/// The version `pattern` captures from `output`: the first group, or the whole match without one.
/// Empty when nothing matches or the pattern is invalid.
pub fn match_version(output: &str, pattern: &str) -> String {
    if pattern.is_empty() || output.is_empty() {
        return String::new();
    }
    let Ok(re) = regex::Regex::new(pattern) else {
        return String::new();
    };
    let Some(caps) = re.captures(output.trim()) else {
        return String::new();
    };
    caps.get(1)
        .or_else(|| caps.get(0))
        .map(|m| m.as_str().to_string())
        .unwrap_or_default()
}
