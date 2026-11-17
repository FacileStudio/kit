use super::fake::*;

const CASES: [(&str, Env, &str); 20] = [
    ("facile:sablier", ALL, "facile:sablier=sablier skip="),
    ("facile:SABLIER", ALL, "facile:sablier=sablier skip="),
    (
        "flathub:org.gimp.GIMP",
        NONE,
        "flathub:org.gimp.GIMP=org.gimp.GIMP skip=",
    ),
    ("nix:ripgrep", ALL, "nix:ripgrep=ripgrep skip="),
    ("nix:anything", UNCHECKED, "nix:anything=anything skip="),
    ("Nuage", ALL, "facile:nuage=nuage skip="),
    (" sablier ", NONE, "facile:sablier=sablier skip="),
    ("firefox", ALL, "flathub:org.mozilla.firefox=Firefox skip="),
    ("FIREFOX", ALL, "flathub:org.mozilla.firefox=Firefox skip="),
    ("Spotify", ALL, "flathub:com.spotify.Client=Spotify skip="),
    ("client", ALL, "flathub:com.spotify.Client=Spotify skip="),
    (
        "org.gimp.GIMP",
        ALL,
        "flathub:org.gimp.GIMP=GNU Image Manipulation Program skip=",
    ),
    (
        "calculator",
        ALL,
        "flathub:org.gnome.Calculator=Calculator | flathub:io.github.calc.Calc=Calculator skip=",
    ),
    ("ripgrep", ALL, "nix:ripgrep=ripgrep skip="),
    (
        "python3Packages.requests",
        ALL,
        "nix:python3Packages.requests=python3Packages.requests skip=",
    ),
    ("firefox", NO_FLATPAK, "nix:firefox=firefox skip=flathub"),
    ("ripgrep", NO_NIX, "none skip=nix"),
    ("firefox", NONE, "none skip=flathub,nix"),
    ("no-such-thing", ALL, "none skip="),
    ("two words", NO_NIX, "none skip="),
];

#[test]
fn resolves_every_case() {
    for (spec, env, want) in CASES {
        let got = run(spec, env).unwrap_or_else(|e| panic!("{spec}: {e}"));
        assert_eq!(describe(&got), want, "{spec}");
    }
}

#[test]
fn serializes_for_json_output() {
    let got = run("firefox", NO_FLATPAK).unwrap();
    assert_eq!(
        serde_json::to_value(&got).unwrap(),
        serde_json::json!({
            "resolution": {"kind": "one", "result": {"source": "nix", "id": "firefox", "name": "firefox"}},
            "skipped": ["flathub"],
        })
    );
    let many = serde_json::to_value(run("calculator", ALL).unwrap()).unwrap();
    assert_eq!(many["resolution"]["kind"], "many");
    assert_eq!(many["resolution"]["result"][1]["id"], "io.github.calc.Calc");
    let none = serde_json::to_value(run("zzz", ALL).unwrap()).unwrap();
    assert_eq!(none["resolution"], serde_json::json!({"kind": "none"}));
}
