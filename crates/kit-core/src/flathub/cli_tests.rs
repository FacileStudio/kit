use tempfile::TempDir;

use super::stub::Stub;
use super::{FlathubError, Flatpak};

type Case = (&'static str, fn(&Flatpak), &'static [&'static str]);

const CASES: [Case; 5] = [
    (
        "ensure_remote",
        |f| f.ensure_remote().unwrap(),
        &[
            "remote-add",
            "--user",
            "--if-not-exists",
            "flathub",
            "https://dl.flathub.org/repo/flathub.flatpakrepo",
        ],
    ),
    (
        "install",
        |f| {
            f.install(&["org.gnome.Calculator", "org.mozilla.firefox"])
                .unwrap()
        },
        &[
            "install",
            "--user",
            "-y",
            "--noninteractive",
            "flathub",
            "org.gnome.Calculator",
            "org.mozilla.firefox",
        ],
    ),
    (
        "uninstall",
        |f| f.uninstall(&["org.mozilla.firefox"]).unwrap(),
        &[
            "uninstall",
            "--user",
            "-y",
            "--noninteractive",
            "org.mozilla.firefox",
        ],
    ),
    (
        "update",
        |f| f.update().unwrap(),
        &["update", "--user", "-y", "--noninteractive"],
    ),
    (
        "list",
        |f| drop(f.list_installed().unwrap()),
        &["list", "--user", "--app", "--columns=application"],
    ),
];

#[test]
fn every_call_sends_the_exact_argv() {
    for (name, call, want) in CASES {
        let stub = Stub::ok();
        call(&stub.flatpak);
        assert_eq!(stub.argv().unwrap(), want, "{name}");
    }
}

#[test]
fn list_installed_parses_ids() {
    let stub = Stub::new("org.gnome.Calculator\norg.mozilla.firefox\n\n", "", 0);
    assert_eq!(
        stub.flatpak.list_installed().unwrap(),
        ["org.gnome.Calculator", "org.mozilla.firefox"]
    );
}

#[test]
fn search_sends_argv_and_parses_pairs() {
    let stub = Stub::new(
        "org.mozilla.firefox\tFirefox\norg.gnome.Epiphany\tWeb Browser\nNo matches found\n",
        "",
        0,
    );
    let found = stub.flatpak.search("-browser").unwrap();
    assert_eq!(
        stub.argv().unwrap(),
        [
            "search",
            "--user",
            "--columns=application,name",
            "--",
            "-browser"
        ]
    );
    assert_eq!(
        found,
        [
            ("org.mozilla.firefox".to_string(), "Firefox".to_string()),
            ("org.gnome.Epiphany".to_string(), "Web Browser".to_string()),
        ]
    );
    assert!(matches!(
        stub.flatpak.search("  "),
        Err(FlathubError::EmptyQuery)
    ));
}

#[test]
fn invalid_or_empty_ids_never_run_flatpak() {
    let stub = Stub::ok();
    let err = stub
        .flatpak
        .install(&["org.gnome.Calculator", "--system"])
        .unwrap_err();
    assert!(matches!(err, FlathubError::InvalidId { ref id, .. } if id == "--system"));
    stub.flatpak.uninstall(&[]).unwrap();
    assert_eq!(stub.argv(), None);
}

#[test]
fn non_zero_exit_returns_code_and_stderr_tail() {
    let noise: String = (1..=15).map(|n| format!("line {n}\n")).collect();
    let stub = Stub::new(
        "",
        &format!("{noise}error: No remote refs found for 'org.x.Y'\n"),
        3,
    );
    match stub.flatpak.install(&["org.x.Y"]).unwrap_err() {
        FlathubError::CommandFailed {
            action,
            code,
            stderr,
        } => {
            assert_eq!(action, "install");
            assert_eq!(code, Some(3));
            assert_eq!(stderr.lines().count(), 10);
            assert!(stderr.starts_with("line 7\n"));
            assert!(stderr.ends_with("error: No remote refs found for 'org.x.Y'"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn missing_flatpak_is_not_available() {
    let empty = TempDir::new().unwrap();
    assert!(matches!(
        Flatpak::locate(empty.path().as_os_str()),
        Err(FlathubError::NotAvailable)
    ));
    let gone = Flatpak::with_binary(empty.path().join("flatpak"));
    assert!(matches!(gone.update(), Err(FlathubError::NotAvailable)));
}
