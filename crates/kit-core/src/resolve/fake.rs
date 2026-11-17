//! Fake backends for the resolve tests.

use super::*;

/// Which backends are installed, and whether `nix:attr` is checked.
#[derive(Clone, Copy)]
pub struct Env {
    pub flatpak: bool,
    pub nix: bool,
    pub check_nix: bool,
}

pub const ALL: Env = Env {
    flatpak: true,
    nix: true,
    check_nix: true,
};
pub const NO_FLATPAK: Env = Env {
    flatpak: false,
    ..ALL
};
pub const NO_NIX: Env = Env { nix: false, ..ALL };
pub const NONE: Env = Env {
    flatpak: false,
    nix: false,
    check_nix: true,
};
pub const UNCHECKED: Env = Env {
    check_nix: false,
    ..NONE
};

const NIX_PKGS: [&str; 3] = ["ripgrep", "firefox", "python3Packages.requests"];

const FLATHUB: [(&str, &str); 7] = [
    ("org.mozilla.firefox", "Firefox"),
    ("org.mozilla.firefox.BaseApp", "Firefox BaseApp"),
    ("org.gnome.Calculator", "Calculator"),
    ("io.github.calc.Calc", "Calculator"),
    ("com.spotify.Client", "Spotify"),
    ("org.gimp.GIMP", "GNU Image Manipulation Program"),
    ("com.example.ripgrepgui", "ripgrep GUI"),
];

/// A catalog holding sablier and nuage.
pub fn catalog(name: &str) -> Result<Option<Tool>, CatalogError> {
    Ok(["sablier", "nuage"]
        .into_iter()
        .find(|t| t.eq_ignore_ascii_case(name))
        .map(|t| Tool {
            name: t.to_string(),
            ..Tool::default()
        }))
}

/// A substring search over [`FLATHUB`], like `flatpak search`.
pub fn hits(query: &str) -> Vec<(String, String)> {
    let q = query.to_lowercase();
    FLATHUB
        .into_iter()
        .filter(|(id, name)| id.to_lowercase().contains(&q) || name.to_lowercase().contains(&q))
        .map(|(id, name)| (id.to_string(), name.to_string()))
        .collect()
}

/// Resolves `spec` against the fakes in `env`.
pub fn run(spec: &str, env: Env) -> Result<Outcome, ResolveError> {
    let search = move |q: &str| match env.flatpak {
        true => Ok(hits(q)),
        false => Err(FlathubError::NotAvailable),
    };
    let exists = move |a: &str| match env.nix {
        true => Ok(NIX_PKGS.contains(&a)),
        false => Err(NixError::NotAvailable("nix".into())),
    };
    let ctx = Context {
        catalog: &catalog,
        flathub_search: &search,
        nix_exists: &exists,
        check_nix: env.check_nix,
    };
    resolve(spec, &ctx)
}

/// Renders an outcome as `source:id=name` entries joined by ` | `, then `skip=` and the skipped backends.
pub fn describe(outcome: &Outcome) -> String {
    let show = |c: &Candidate| format!("{}:{}={}", c.source, c.id, c.name);
    let found = match &outcome.resolution {
        Resolution::One(c) => show(c),
        Resolution::Many(cs) => cs.iter().map(show).collect::<Vec<_>>().join(" | "),
        Resolution::None => "none".to_string(),
    };
    let skipped: Vec<&str> = outcome.skipped.iter().map(|s| s.as_str()).collect();
    format!("{found} skip={}", skipped.join(","))
}
