use std::fs;
use std::io;
use std::path::Path;

/// Replaces the tool's marked section in a shared agent file and keeps everything outside the markers.
pub(crate) fn inject_block(path: &Path, skill: &str, body: &[u8]) -> io::Result<()> {
    let start = format!("<!-- {skill}:start -->");
    let end = format!("<!-- {skill}:end -->");
    let kept = kept_lines(path, &start, &end)?;
    let mut out = String::new();
    if !kept.is_empty() {
        out.push_str(kept.join("\n").trim_end_matches('\n'));
        out.push_str("\n\n");
    }
    out.push_str(&start);
    out.push('\n');
    let text = String::from_utf8_lossy(body);
    out.push_str(&text);
    if !text.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&end);
    out.push('\n');
    fs::write(path, out)
}

fn kept_lines(path: &Path, start: &str, end: &str) -> io::Result<Vec<String>> {
    let existing = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let mut kept = Vec::new();
    let mut skipping = false;
    for line in existing.lines() {
        if line == start {
            skipping = true;
        } else if line == end {
            skipping = false;
        } else if !skipping {
            kept.push(line.to_string());
        }
    }
    Ok(kept)
}
