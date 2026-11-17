use super::{validate_id, FlathubError};

#[test]
fn accepts_valid_ids() {
    let long_last = format!("org.example.{}", "a".repeat(243));
    for id in [
        "org.gnome.Calculator",
        "com.github.tchx84.Flatseal",
        "org.mozilla.firefox",
        "io.github.some_user.My-App",
        "org.freedesktop.Platform.GL.default",
        "_x.y.z",
        long_last.as_str(),
    ] {
        assert!(validate_id(id).is_ok(), "{id} should be valid");
    }
}

#[test]
fn rejects_invalid_ids() {
    let too_long = format!("org.example.{}", "a".repeat(244));
    for id in [
        "",
        "firefox",
        "org.mozilla",
        ".org.mozilla.firefox",
        "org.mozilla.firefox.",
        "org..mozilla.firefox",
        "org.1mozilla.firefox",
        "org.mo-zilla.firefox",
        "org.mozilla.fire fox",
        "org.mozilla.firefox;rm",
        "org.mozilla.firefox/x86_64",
        "-org.mozilla.firefox",
        "--user.a.b",
        "org.mozilla.fïrefox",
        too_long.as_str(),
    ] {
        assert!(
            matches!(validate_id(id), Err(FlathubError::InvalidId { .. })),
            "{id:?} should be rejected"
        );
    }
}
