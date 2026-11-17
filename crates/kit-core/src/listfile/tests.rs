use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use super::*;

fn store(dir: &tempfile::TempDir) -> ListStore {
    ListStore::new(dir.path().join("kit.yml")).with_seed(dir.path().join("missing-seed.yml"))
}

#[test]
fn round_trip_keeps_order() {
    let dir = tempfile::tempdir().unwrap();
    let store = store(&dir);
    store
        .edit(|file| {
            file.add(Source::Nix, "ripgrep");
            file.add(Source::Facile, "sablier");
            file.add(Source::Nix, "fd");
            file.add(Source::Flathub, "org.mozilla.firefox");
        })
        .unwrap();
    let text = std::fs::read_to_string(store.path()).unwrap();
    assert!(text.starts_with("version: 1\n"), "{text}");
    let file = store.read().unwrap();
    assert_eq!(file.nix, ["ripgrep", "fd"]);
    assert_eq!(file.entries(Source::Facile), ["sablier"]);
    assert_eq!(file.flathub, ["org.mozilla.firefox"]);
    store
        .edit(|file| assert!(file.remove(Source::Nix, "ripgrep")))
        .unwrap();
    assert_eq!(store.read().unwrap().nix, ["fd"]);
}

#[test]
fn dedupes_on_add_and_on_read() {
    let mut file = KitFile::default();
    assert!(file.add(Source::Nix, "fd"));
    assert!(!file.add(Source::Nix, "fd"));
    assert!(!file.remove(Source::Nix, "absent"));
    assert_eq!(file.nix, ["fd"]);
    let dir = tempfile::tempdir().unwrap();
    let store = store(&dir);
    std::fs::write(store.path(), "version: 1\nnix: [b, a, b, c, a]\n").unwrap();
    assert_eq!(store.read().unwrap().nix, ["b", "a", "c"]);
}

#[test]
fn missing_file_copies_seed() {
    let dir = tempfile::tempdir().unwrap();
    let seed = dir.path().join("seed.yml");
    std::fs::write(&seed, "version: 1\nflathub: [org.gnome.Calculator]\n").unwrap();
    let store = ListStore::new(dir.path().join("sub/kit.yml")).with_seed(&seed);
    assert_eq!(store.read().unwrap().flathub, ["org.gnome.Calculator"]);
    store
        .edit(|file| file.add(Source::Facile, "nuage"))
        .unwrap();
    let file = KitFile::read(store.path()).unwrap();
    assert_eq!(file.flathub, ["org.gnome.Calculator"]);
    assert_eq!(file.facile, ["nuage"]);
    let empty = ListStore::new(dir.path().join("none.yml")).with_seed(dir.path().join("nope.yml"));
    assert_eq!(empty.read().unwrap(), KitFile::default());
}

#[test]
fn rejects_unknown_key_and_version() {
    let dir = tempfile::tempdir().unwrap();
    let store = store(&dir);
    std::fs::write(store.path(), "version: 1\nbrew: [wget]\n").unwrap();
    let err = store.read().unwrap_err();
    assert!(matches!(&err, ListError::UnknownKey { key, .. } if key == "brew"));
    assert!(err.to_string().contains("`brew`"), "{err}");
    std::fs::write(store.path(), "version: 2\n").unwrap();
    assert!(matches!(
        store.read().unwrap_err(),
        ListError::Version { found: 2, .. }
    ));
    std::fs::write(store.path(), "- a\n").unwrap();
    assert!(matches!(
        store.read().unwrap_err(),
        ListError::NotMapping { .. }
    ));
}

#[test]
fn source_prefix_parse_and_display() {
    assert_eq!(
        Source::split_spec("nix:hello").unwrap(),
        (Some(Source::Nix), "hello")
    );
    assert_eq!(Source::split_spec("ripgrep").unwrap(), (None, "ripgrep"));
    assert!(
        matches!(Source::split_spec("brew:wget"), Err(ListError::UnknownSource(s)) if s == "brew")
    );
    for source in Source::ALL {
        assert_eq!(source.to_string().parse::<Source>().unwrap(), source);
    }
}

#[test]
fn second_editor_blocks_until_lock_released() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kit.yml");
    let (locked_tx, locked_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let first_path = path.clone();
    let first = thread::spawn(move || {
        KitFile::edit(&first_path, |file| {
            file.add(Source::Nix, "first");
            locked_tx.send(()).unwrap();
            release_rx.recv().unwrap();
        })
        .unwrap()
    });
    locked_rx.recv().unwrap();
    let done = Arc::new(AtomicBool::new(false));
    let second_done = Arc::clone(&done);
    let second_path = path.clone();
    let second = thread::spawn(move || {
        KitFile::edit(&second_path, |file| file.add(Source::Nix, "second")).unwrap();
        second_done.store(true, Ordering::SeqCst);
    });
    thread::sleep(Duration::from_millis(200));
    assert!(
        !done.load(Ordering::SeqCst),
        "second edit ran while the lock was held"
    );
    release_tx.send(()).unwrap();
    first.join().unwrap();
    second.join().unwrap();
    assert_eq!(KitFile::read(&path).unwrap().nix, ["first", "second"]);
}
