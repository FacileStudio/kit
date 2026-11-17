use std::fs;
use std::path::Path;

use super::super::{on_path, tilde};
use crate::catalog::{sources_path, sources_paths_in};

#[test]
fn sources_paths_order_and_fallback() {
    let home = Path::new("/h");
    let paths = sources_paths_in(home, &home.join(".config/kit"));
    let want: Vec<_> = [
        "/h/.config/kit/config.yml",
        "/h/.config/kit/config.yaml",
        "/h/.facile.yml",
        "/h/.facile.yaml",
        "/h/.config/facile/config.yml",
        "/h/.config/facile/config.yaml",
        "/h/.config/facile/sources.yml",
        "/h/.config/facile/sources.yaml",
    ]
    .iter()
    .map(Path::new)
    .collect();
    assert_eq!(paths, want);
}

#[test]
fn sources_paths_respects_xdg() {
    let paths = sources_paths_in(Path::new("/h"), Path::new("/h/xdg/kit"));
    assert_eq!(paths[0], Path::new("/h/xdg/kit/config.yml"));
    assert_eq!(paths[4], Path::new("/h/xdg/facile/config.yml"));
    assert_eq!(paths[5], Path::new("/h/xdg/facile/config.yaml"));
}

#[test]
fn sources_path_prefers_config_yml_when_no_dotfile() {
    let home = tempfile::tempdir().unwrap();
    let paths = sources_paths_in(home.path(), &home.path().join(".config/kit"));
    let legacy = home.path().join(".config/facile");
    fs::create_dir_all(&legacy).unwrap();
    fs::write(legacy.join("sources.yml"), "").unwrap();
    fs::write(legacy.join("config.yml"), "").unwrap();
    assert_eq!(sources_path(&paths), Some(legacy.join("config.yml")));
}

#[test]
fn sources_path_finds_yaml_variant() {
    let home = tempfile::tempdir().unwrap();
    let paths = sources_paths_in(home.path(), &home.path().join(".config/kit"));
    let legacy = home.path().join(".config/facile");
    fs::create_dir_all(&legacy).unwrap();
    fs::write(legacy.join("config.yaml"), "").unwrap();
    assert_eq!(sources_path(&paths), Some(legacy.join("config.yaml")));
}

#[test]
fn tilde_shortens_the_home_directory() {
    let home = Path::new("/home/me");
    assert_eq!(
        tilde(Path::new("/home/me/.local/bin"), home),
        "~/.local/bin"
    );
    assert_eq!(tilde(Path::new("/home/me"), home), "~");
    assert_eq!(tilde(Path::new("/home/meow/bin"), home), "/home/meow/bin");
}

#[test]
fn on_path_matches_whole_entries() {
    let path = std::ffi::OsString::from("/usr/bin:/home/me/.local/bin");
    assert!(on_path(Path::new("/home/me/.local/bin"), &path));
    assert!(!on_path(Path::new("/home/me/.local"), &path));
}
