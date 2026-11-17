use std::fs;
use std::path::Path;

use super::super::{from_homebrew, outdated, owner_of, real_path, version_of, Owner};

#[test]
fn from_homebrew_matches_install_paths_only() {
    let cases = [
        ("/opt/homebrew/Caskroom/kit/0.8.0/kit", true),
        ("/usr/local/Cellar/kit/0.8.0/bin/kit", true),
        ("/home/linuxbrew/.linuxbrew/bin/kit", true),
        ("/Users/someone/.local/bin/kit", false),
        ("/usr/local/bin/kit", false),
        ("/Users/someone/Projects/homebrew-tap/x/kit", false),
    ];
    for (path, want) in cases {
        assert_eq!(from_homebrew(path), want, "{path}");
    }
}

#[test]
fn outdated_is_an_ordering_never_an_inequality() {
    let cases = [
        ("behind", "0.7.0", "0.8.0", true),
        ("current", "0.8.0", "0.8.0", false),
        ("commit sha", "edf2b6f", "0.25.0", false),
        ("dev build", "dev", "0.8.0", false),
        ("unresolved tag", "0.7.0", "", false),
        ("nothing installed", "", "0.8.0", false),
        (
            "prerelease is behind its release",
            "0.9.0-rc1",
            "0.9.0",
            true,
        ),
        (
            "release is not behind its prerelease",
            "0.9.0",
            "0.9.0-rc1",
            false,
        ),
        ("ahead of a stale cache", "0.9.0", "0.8.0", false),
        ("ahead by a minor", "0.10.0", "0.9.0", false),
        ("ahead by a patch", "0.9.1", "0.9.0", false),
        ("double digit minor", "0.9.0", "0.10.0", true),
        ("double digit patch", "1.2.9", "1.2.10", true),
    ];
    for (name, have, latest, want) in cases {
        assert_eq!(outdated(have, latest), want, "{name}");
    }
}

#[test]
fn version_of_reads_both_line_shapes() {
    assert_eq!(version_of("opus 1.2.3"), "1.2.3");
    assert_eq!(version_of("opus version 1.2.3"), "1.2.3");
    assert_eq!(version_of("1.2.3"), "1.2.3");
}

#[test]
fn real_path_sees_through_a_symlinked_directory() {
    let root = tempfile::tempdir().unwrap();
    let actual = root.path().join("actual");
    fs::create_dir_all(&actual).unwrap();
    fs::write(actual.join("opus"), "#!/bin/sh\n").unwrap();
    let link = root.path().join("linked");
    std::os::unix::fs::symlink(&actual, &link).unwrap();
    assert_eq!(
        real_path(&link.join("opus")),
        real_path(&actual.join("opus"))
    );
}

#[test]
fn owner_of_names_who_placed_a_binary() {
    let root = tempfile::tempdir().unwrap();
    let bin = root.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::write(bin.join("opus"), "").unwrap();
    assert_eq!(owner_of(&bin.join("opus"), &bin), Owner::Kit);
    let mise = root.path().join("share/mise/installs/opus/1.0.0");
    fs::create_dir_all(&mise).unwrap();
    fs::write(mise.join("opus"), "").unwrap();
    std::os::unix::fs::symlink(mise.join("opus"), bin.join("mised")).unwrap();
    assert_eq!(owner_of(&bin.join("mised"), &bin), Owner::Mise);
    assert_eq!(
        owner_of(Path::new("/nix/store/abc-opus/bin/opus"), &bin),
        Owner::Nix
    );
    assert_eq!(
        owner_of(Path::new("/opt/homebrew/Caskroom/kit/1/kit"), &bin),
        Owner::Homebrew
    );
    assert_eq!(
        owner_of(Path::new("/srv/elsewhere/opus"), &bin),
        Owner::Other
    );
}

#[test]
fn foreign_owners_name_their_update_command() {
    assert_eq!(
        Owner::Homebrew.update_command("kit"),
        "brew upgrade --cask kit"
    );
    assert_eq!(Owner::Mise.update_command("opus"), "mise upgrade");
    assert_eq!(Owner::Kit.update_command("kit"), "kit self-update");
    assert_eq!(Owner::Kit.update_command("opus"), "kit update opus");
    assert!(Owner::Nix.is_foreign() && !Owner::Kit.is_foreign() && !Owner::Other.is_foreign());
}
