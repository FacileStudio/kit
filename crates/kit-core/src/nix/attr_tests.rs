use super::attr::{env_expr, exists_expr, normalize, parse_rev};
use super::{validate_attr, NixError};

#[test]
fn validate_attr_table() {
    let cases: &[(&str, bool)] = &[
        ("ripgrep", true),
        ("python3Packages.requests", true),
        ("_7zz", true),
        ("gnome.gnome-tweaks", true),
        ("nodePackages_latest.pnpm", true),
        ("", false),
        (".hello", false),
        ("hello.", false),
        ("a..b", false),
        ("7zip", false),
        ("-rf", false),
        ("hello world", false),
        ("hello\"", false),
        ("a\"; builtins.exec x; \"", false),
        ("${builtins.readFile /etc/shadow}", false),
        ("$HOME", false),
        ("foo;bar", false),
        ("foo\nbar", false),
        ("foo'", false),
        ("foo/bar", false),
        ("héllo", false),
    ];
    for (attr, ok) in cases {
        assert_eq!(validate_attr(attr).is_ok(), *ok, "{attr:?}");
    }
    assert!(validate_attr(&"a".repeat(201)).is_err());
}

#[test]
fn normalize_sorts_dedups_and_rejects() {
    let got = normalize(&["b".into(), "a".into(), "b".into()]).unwrap();
    assert_eq!(got, vec!["a", "b"]);
    assert!(
        matches!(normalize(&["ok".into(), "no pe".into()]), Err(NixError::InvalidAttr(a)) if a == "no pe")
    );
}

#[test]
fn parse_rev_reads_locked_then_revision() {
    assert_eq!(
        parse_rev(r#"{"locked":{"rev":"aa11"},"revision":"bb22"}"#).unwrap(),
        "aa11"
    );
    assert_eq!(
        parse_rev(r#"{"locked":{},"revision":"bb22"}"#).unwrap(),
        "bb22"
    );
    assert!(parse_rev(r#"{"locked":{"path":"/x"}}"#).is_err());
    assert!(parse_rev(r#"{"revision":"a\"b"}"#).is_err());
    assert!(parse_rev("not json").is_err());
}

#[test]
fn env_expr_is_exact() {
    let expr = env_expr("abc", &["hello".into(), "python3Packages.foo".into()]).unwrap();
    assert_eq!(
        expr,
        "let pkgs = (builtins.getFlake \"nixpkgs\").legacyPackages.${builtins.currentSystem}; in \
         pkgs.buildEnv { name = \"kit-env\"; paths = [ pkgs.hello pkgs.python3Packages.foo \
         (pkgs.writeTextDir \"share/kit/env.json\" (builtins.toJSON { rev = \"abc\"; \
         attrs = [ \"hello\" \"python3Packages.foo\" ]; })) ]; }"
    );
    assert!(env_expr("abc", &["x;y".into()]).is_err());
    assert!(env_expr("a\"b", &[]).is_err());
}

#[test]
fn exists_expr_is_exact() {
    assert_eq!(
        exists_expr("python3Packages.foo").unwrap(),
        "let pkgs = (builtins.getFlake \"nixpkgs\").legacyPackages.${builtins.currentSystem}; in \
         pkgs ? python3Packages.foo"
    );
    assert!(exists_expr("a b").is_err());
}
