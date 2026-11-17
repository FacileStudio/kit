//! Flatpak application ID validation, after `flatpak_is_valid_name` in flatpak's `common/flatpak-ref-utils.c`.

use super::FlathubError;

const MAX_LEN: usize = 255;
const MIN_SEGMENTS: usize = 3;

/// Checks that `id` is a reverse-DNS Flatpak app ID such as `org.gnome.Calculator`.
///
/// Flatpak requires three or more non-empty segments of `[A-Za-z0-9_-]`, none starting with a digit,
/// at most 255 bytes, and `-` only in the last segment.
pub fn validate_id(id: &str) -> Result<(), FlathubError> {
    let fail = |reason| {
        Err(FlathubError::InvalidId {
            id: id.to_string(),
            reason,
        })
    };
    if id.is_empty() {
        return fail("it is empty");
    }
    if id.len() > MAX_LEN {
        return fail("it is longer than 255 characters");
    }
    let segments: Vec<&str> = id.split('.').collect();
    if segments.len() < MIN_SEGMENTS {
        return fail("it needs at least three dot-separated segments, like org.example.App");
    }
    let last = segments.len() - 1;
    for (index, segment) in segments.iter().enumerate() {
        if let Some(reason) = segment_error(segment, index == last) {
            return fail(reason);
        }
    }
    Ok(())
}

fn segment_error(segment: &str, last: bool) -> Option<&'static str> {
    let Some(first) = segment.chars().next() else {
        return Some("a segment is empty");
    };
    if first.is_ascii_digit() {
        return Some("a segment starts with a digit");
    }
    for c in segment.chars() {
        if c == '-' && !last {
            return Some("only the last segment may contain -");
        }
        if !(c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Some("only letters, digits, _ and - are allowed");
        }
    }
    None
}
