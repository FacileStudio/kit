use super::fake::*;
use super::*;

type ErrCheck = fn(&ResolveError) -> bool;

const CASES: [(&str, Env, ErrCheck); 8] = [
    ("", ALL, |e| matches!(e, ResolveError::Empty)),
    ("nix:", ALL, |e| matches!(e, ResolveError::Empty)),
    ("brew:wget", ALL, |e| matches!(e, ResolveError::Spec(_))),
    ("facile:firefox", ALL, |e| {
        matches!(e, ResolveError::UnknownTool(_))
    }),
    ("flathub:firefox", ALL, |e| {
        matches!(e, ResolveError::Flathub(FlathubError::InvalidId { .. }))
    }),
    ("nix:rm -rf", ALL, |e| {
        matches!(e, ResolveError::Nix(NixError::InvalidAttr(_)))
    }),
    ("nix:nosuchpkg", ALL, |e| {
        matches!(e, ResolveError::UnknownAttr(_))
    }),
    ("nix:ripgrep", NO_NIX, |e| {
        matches!(e, ResolveError::Nix(NixError::NotAvailable(_)))
    }),
];

#[test]
fn rejects_bad_specs() {
    for (spec, env, check) in CASES {
        let err = run(spec, env).err();
        assert!(err.as_ref().is_some_and(check), "{spec}: {err:?}");
    }
}

#[test]
fn backend_failures_propagate() {
    let broken =
        |_: &str| -> Result<Vec<(String, String)>, FlathubError> { Err(FlathubError::EmptyQuery) };
    let empty = |_: &str| -> Result<Vec<(String, String)>, FlathubError> { Ok(Vec::new()) };
    let exists = |_: &str| -> Result<bool, NixError> { Err(NixError::BadOutput("x".into())) };
    let ctx = Context {
        catalog: &catalog,
        flathub_search: &broken,
        nix_exists: &exists,
        check_nix: true,
    };
    assert!(matches!(
        resolve("firefox", &ctx),
        Err(ResolveError::Flathub(_))
    ));
    let ctx = Context {
        flathub_search: &empty,
        ..ctx
    };
    assert!(matches!(
        resolve("firefox", &ctx),
        Err(ResolveError::Nix(_))
    ));
}

#[test]
fn catalog_failure_propagates() {
    let failing = |_: &str| -> Result<Option<Tool>, CatalogError> { Err(CatalogError::NoTools) };
    let ctx = Context {
        catalog: &failing,
        flathub_search: &|q: &str| Ok(hits(q)),
        nix_exists: &|_: &str| Ok(false),
        check_nix: true,
    };
    assert!(matches!(
        resolve("sablier", &ctx),
        Err(ResolveError::Catalog(_))
    ));
}
