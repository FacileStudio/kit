use anyhow::anyhow;

/// Splits arguments on commas and whitespace, dropping empties and case-insensitive repeats.
pub fn split_names(args: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let parts = args
        .iter()
        .flat_map(|a| a.split(|c: char| c == ',' || c.is_whitespace()));
    for part in parts.filter(|p| !p.is_empty()) {
        if !out.iter().any(|seen| seen.eq_ignore_ascii_case(part)) {
            out.push(part.to_string());
        }
    }
    out
}

/// Takes kit out of an update's names: whether kit updates, the other names, and whether any name was given.
///
/// `kit update kit` updates kit alone, so naming only kit must not widen to everything listed.
pub fn split_self(args: &[String]) -> (bool, Vec<String>, bool) {
    let names = split_names(args);
    let named = !names.is_empty();
    let (me, rest): (Vec<String>, Vec<String>) = names.into_iter().partition(|n| is_self(n));
    (!named || !me.is_empty(), rest, named)
}

/// True for the names that mean kit itself.
pub fn is_self(name: &str) -> bool {
    name.eq_ignore_ascii_case("kit") || name.eq_ignore_ascii_case("facile:kit")
}

/// The answer to adding or removing kit: it only replaces itself.
pub fn self_error() -> anyhow::Error {
    anyhow!(
        "kit does not add or remove itself: `kit self-update` replaces the running binary in place"
    )
}
