use std::fs;
use std::path::Path;

use super::super::inject::inject_block;
use super::super::Quiet;
use super::{backend, offline};
use crate::catalog::Tool;

const BODY: &str = "# journal: Facile centralized logging\n\nBinary: `journal`\n";

fn journal() -> Tool {
    Tool {
        name: "journal".into(),
        skill: "journal".into(),
        ..Tool::default()
    }
}

fn register_in(home: &Path, body: &str) {
    let work = home.join("work");
    fs::create_dir_all(work.join("src/integrations")).unwrap();
    fs::write(work.join("src/integrations/SKILL.md"), body).unwrap();
    let (b, _) = backend(home, offline);
    b.register_skill(&journal(), &work, &Quiet);
}

#[test]
fn register_skill_writes_the_pi_target() {
    let home = tempfile::tempdir().unwrap();
    let skills = home.path().join(".local/share/mycelium/skills");
    fs::create_dir_all(&skills).unwrap();
    register_in(home.path(), BODY);
    assert_eq!(fs::read_to_string(skills.join("journal.md")).unwrap(), BODY);
}

#[test]
fn register_skill_falls_back_to_legacy_pi_target() {
    let home = tempfile::tempdir().unwrap();
    let skills = home.path().join(".mycelium/skills");
    fs::create_dir_all(&skills).unwrap();
    register_in(home.path(), BODY);
    assert_eq!(fs::read_to_string(skills.join("journal.md")).unwrap(), BODY);
}

#[test]
fn no_mycelium_skills_dir_does_not_create_one() {
    let home = tempfile::tempdir().unwrap();
    register_in(home.path(), "x");
    assert!(!home.path().join(".mycelium").exists());
    assert!(!home.path().join(".local/share/mycelium").exists());
}

#[test]
fn inject_block_replaces_only_the_marked_section() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("AGENTS.md");
    fs::write(
        &path,
        "mine\n<!-- journal:start -->\nold\n<!-- journal:end -->\n",
    )
    .unwrap();
    inject_block(&path, "journal", b"new").unwrap();
    inject_block(&path, "journal", b"newer\n").unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(
        text,
        "mine\n\n<!-- journal:start -->\nnewer\n<!-- journal:end -->\n"
    );
}
