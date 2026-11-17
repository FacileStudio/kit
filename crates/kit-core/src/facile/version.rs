#[derive(Debug, Clone, PartialEq, Eq)]
struct Release {
    num: [u64; 3],
    pre: String,
}

fn parse_release(v: &str) -> Option<Release> {
    let (core, pre) = match v.find(['-', '+']) {
        Some(i) => (&v[..i], &v[i + 1..]),
        None => (v, ""),
    };
    if v.len() > core.len() && pre.is_empty() {
        return None;
    }
    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let mut num = [0u64; 3];
    for (slot, part) in num.iter_mut().zip(&parts) {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        *slot = part.parse().ok()?;
    }
    Some(Release {
        num,
        pre: pre.to_string(),
    })
}

impl Release {
    fn before(&self, other: &Release) -> bool {
        if self.num != other.num {
            return self.num < other.num;
        }
        if self.pre.is_empty() != other.pre.is_empty() {
            return !self.pre.is_empty();
        }
        self.pre < other.pre
    }
}

/// True for a plain `MAJOR.MINOR.PATCH`, optionally with a prerelease or build suffix.
pub fn is_semver(v: &str) -> bool {
    parse_release(v).is_some()
}

/// True when `have` is strictly older than `latest`. Anything that is not a plain semver answers no.
///
/// It is an ordering, never an inequality: a day-old cached tag must not show a newer binary as behind.
pub fn outdated(have: &str, latest: &str) -> bool {
    match (parse_release(have), parse_release(latest)) {
        (Some(mine), Some(newest)) => mine.before(&newest),
        _ => false,
    }
}

/// The version part of a `{tool} {version}` or `{tool} version {version}` line.
pub fn version_of(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 && parts[1] == "version" {
        return parts[2].to_string();
    }
    match line.split_once(' ') {
        Some((_, ver)) => ver.to_string(),
        None => line.to_string(),
    }
}
