mod common;

use common::{json, stderr, stdout, Sandbox};

#[test]
fn add_then_del_round_trips_the_list_file() {
    let sb = Sandbox::new().with_backends();
    let out = sb.run(&["add", "flathub:org.gimp.GIMP", "nix:ripgrep"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        sb.read_file(),
        "version: 1\nfacile: []\nnix:\n- ripgrep\nflathub:\n- org.gimp.GIMP\n"
    );
    assert!(
        sb.log()
            .contains("flatpak install --user -y --noninteractive flathub org.gimp.GIMP"),
        "{}",
        sb.log()
    );
    assert!(sb.log().contains("--profile"), "nix sync ran: {}", sb.log());
    let out = sb.run(&["del", "GIMP", "ripgrep"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        sb.read_file(),
        "version: 1\nfacile: []\nnix: []\nflathub: []\n"
    );
    assert!(
        sb.log()
            .contains("flatpak uninstall --user -y --noninteractive org.gimp.GIMP"),
        "{}",
        sb.log()
    );
}

#[test]
fn add_takes_comma_lists_and_dedupes() {
    let sb = Sandbox::new().with_backends();
    let out = sb.run(&[
        "add",
        "ripgrep,nix:ripgrep",
        "org.gimp.GIMP,ripgrep",
        "--json",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        sb.read_file(),
        "version: 1\nfacile: []\nnix:\n- ripgrep\nflathub:\n- org.gimp.GIMP\n"
    );
    let doc = json(&out);
    let added = doc["added"].as_array().unwrap();
    assert_eq!(added.len(), 2, "{doc}");
    assert_eq!(added[0]["source"], "flathub");
    assert_eq!(added[1]["state"], "installed");
}

#[test]
fn many_matches_without_a_terminal_fail_with_the_candidates() {
    let sb = Sandbox::new().with_backends();
    let out = sb.run(&["add", "calculator"]);
    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(
        err.contains("`calculator` matches several packages"),
        "{err}"
    );
    assert!(
        err.contains("flathub:org.gnome.Calculator") && err.contains("flathub:io.github.calc.Calc"),
        "{err}"
    );
    let out = sb.run(&["add", "calculator", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    let doc = json(&out);
    assert_eq!(doc["candidates"].as_array().unwrap().len(), 2, "{doc}");
    assert_eq!(doc["candidates"][0]["id"], "org.gnome.Calculator");
    assert!(
        sb.read_file().is_empty(),
        "nothing is written for an ambiguous name"
    );
}

#[test]
fn a_bare_name_without_nix_names_kit_setup_nix() {
    let sb = Sandbox::new();
    let out = sb.run(&["add", "ripgrep"]);
    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(err.contains("no package named `ripgrep`"), "{err}");
    assert!(err.contains("kit setup nix"), "{err}");
    assert!(sb.read_file().is_empty());
}

#[test]
fn kit_refuses_to_add_or_remove_itself() {
    let sb = Sandbox::new();
    for cmd in ["add", "del"] {
        let out = sb.run(&[cmd, "kit"]);
        assert_eq!(out.status.code(), Some(1));
        assert!(stderr(&out).contains("kit self-update"), "{}", stderr(&out));
    }
}

#[test]
fn del_of_an_unlisted_name_says_so() {
    let sb = Sandbox::new();
    let out = sb.run(&["del", "nix:hello"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("`nix:hello` is not in the list file"),
        "{}",
        stderr(&out)
    );
    assert!(stdout(&out).is_empty());
}
