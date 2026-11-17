mod common;

use common::{json, stderr, stdout, Sandbox};

const LISTED: &str = "version: 1\nfacile: []\nnix:\n- ripgrep\nflathub:\n- org.gimp.GIMP\n";

#[test]
fn sync_quiet_skips_missing_backends_with_one_hint_each() {
    let sb = Sandbox::new();
    sb.write_file(LISTED);
    let out = sb.run(&["sync", "--quiet"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        stdout(&out).is_empty(),
        "quiet prints nothing on stdout: {}",
        stdout(&out)
    );
    let err = stderr(&out);
    assert_eq!(err.matches("kit setup nix").count(), 1, "{err}");
    assert_eq!(err.matches("install Flatpak").count(), 1, "{err}");
}

#[test]
fn sync_installs_only_what_is_missing() {
    let sb = Sandbox::new().with_backends();
    sb.write_file(LISTED);
    assert!(sb.run(&["sync"]).status.success());
    let first = sb.log();
    assert!(
        first.contains("flatpak install") && first.contains("--profile"),
        "{first}"
    );
    let out = sb.run(&["sync"]);
    assert!(out.status.success());
    assert!(
        stdout(&out).contains("Everything listed is installed"),
        "{}",
        stdout(&out)
    );
    assert_eq!(
        sb.log().matches("flatpak install").count(),
        1,
        "{}",
        sb.log()
    );
    assert_eq!(sb.log().matches("--profile").count(), 1, "{}", sb.log());
}

#[test]
fn check_exits_one_on_drift_and_zero_when_clean() {
    let sb = Sandbox::new().with_backends();
    sb.write_file(LISTED);
    let out = sb.run(&["check", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    let doc = json(&out);
    assert_eq!(doc["missing"].as_array().unwrap().len(), 2, "{doc}");
    for key in ["extra", "outdated", "skipped"] {
        assert!(doc[key].is_array(), "{key} missing from {doc}");
    }
    assert!(sb.run(&["sync"]).status.success());
    let out = sb.run(&["check"]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(stdout(&out).contains("This machine matches"));
}

#[test]
fn check_reports_an_unlisted_flatpak_as_extra() {
    let sb = Sandbox::new().with_backends();
    std::fs::write(sb.dir.path().join("installed"), "org.other.App\n").unwrap();
    let out = sb.run(&["check"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stdout(&out).contains("- flathub:org.other.App"),
        "{}",
        stdout(&out)
    );
    assert!(sb.run(&["dump", "--yes"]).status.success());
    assert!(
        sb.read_file().contains("- org.other.App"),
        "{}",
        sb.read_file()
    );
    assert_eq!(sb.run(&["check"]).status.code(), Some(0));
}

#[test]
fn clean_asks_first_and_refuses_without_a_terminal() {
    let sb = Sandbox::new().with_backends();
    std::fs::write(sb.dir.path().join("installed"), "org.other.App\n").unwrap();
    let out = sb.run(&["clean"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("pass --yes"), "{}", stderr(&out));
    assert!(!sb.log().contains("uninstall"), "{}", sb.log());
    assert!(sb.run(&["clean", "--yes"]).status.success());
    assert!(
        sb.log()
            .contains("flatpak uninstall --user -y --noninteractive org.other.App"),
        "{}",
        sb.log()
    );
}

#[test]
fn list_json_has_one_section_per_source_with_kit_first() {
    let sb = Sandbox::new().with_backends();
    sb.write_file(LISTED);
    let out = sb.run(&["list", "--json"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let doc = json(&out);
    assert_eq!(doc["facile"][0]["name"], "kit");
    assert_eq!(doc["facile"][0]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(doc["nix"][0]["name"], "ripgrep");
    assert_eq!(doc["flathub"][0]["installed"], false);
}

#[test]
fn version_prints_one_line_both_ways() {
    let sb = Sandbox::new();
    let want = format!("kit {}\n", env!("CARGO_PKG_VERSION"));
    assert_eq!(stdout(&sb.run(&["--version"])), want);
    assert_eq!(stdout(&sb.run(&["version"])), want);
    assert_eq!(
        json(&sb.run(&["version", "--json"]))["version"],
        env!("CARGO_PKG_VERSION")
    );
}

#[test]
fn undo_is_nix_only() {
    let sb = Sandbox::new();
    let out = sb.run(&["undo", "flathub"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("undo is not available for flathub"),
        "{}",
        stderr(&out)
    );
    let out = sb.run(&["undo"]);
    assert!(stderr(&out).contains("kit setup nix"), "{}", stderr(&out));
}
