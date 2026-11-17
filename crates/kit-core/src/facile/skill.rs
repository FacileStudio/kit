use std::fs;
use std::path::{Path, PathBuf};

use super::inject::inject_block;
use super::pathutil::which;
use super::{Backend, Reporter};
use crate::catalog::Tool;

impl Backend {
    /// Teaches the AI agents on this machine about the tool. Best effort: a missing skill never fails an install.
    pub(crate) fn register_skill(&self, tool: &Tool, work: &Path, r: &dyn Reporter) {
        if tool.skill.is_empty() {
            return;
        }
        let claude = self.have("claude");
        let codex = self.have("codex");
        let mycelium = self.mycelium_skills_dir();
        if !claude && !codex && mycelium.is_none() {
            return;
        }
        let Some(body) = self.skill_body(tool, work).filter(|b| !b.is_empty()) else {
            return;
        };
        let home = &self.env.home;
        let written = [
            (
                claude && write_claude(home, &tool.skill, &body),
                "Claude Code skill installed",
            ),
            (
                codex && write_codex(home, &tool.skill, &body),
                "Codex skill installed",
            ),
            (
                mycelium.is_some_and(|dir| write_pi(&dir, &tool.skill, &body)),
                "pi skill installed",
            ),
        ];
        for (_, message) in written.iter().filter(|(ok, _)| *ok) {
            r.success(message);
        }
    }

    /// The mycelium skills directory the pi extension reads: XDG data first, then `~/.mycelium/skills`.
    ///
    /// It is never created: its presence is the signal that the machine uses it.
    pub fn mycelium_skills_dir(&self) -> Option<PathBuf> {
        let base = self
            .env
            .data_home
            .clone()
            .unwrap_or_else(|| self.env.home.join(".local").join("share"));
        [
            base.join("mycelium").join("skills"),
            self.env.home.join(".mycelium").join("skills"),
        ]
        .into_iter()
        .find(|dir| dir.is_dir())
    }

    fn have(&self, bin: &str) -> bool {
        which(bin, &self.env.path).is_some()
    }

    fn skill_body(&self, tool: &Tool, work: &Path) -> Option<Vec<u8>> {
        let local = work.join("src").join("integrations").join("SKILL.md");
        if let Ok(body) = fs::read(local) {
            return Some(body);
        }
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/integrations/SKILL.md",
            tool.repo, tool.branch
        );
        self.github.net.get(&url, &[]).ok()
    }
}

fn write_claude(home: &Path, skill: &str, body: &[u8]) -> bool {
    let dir = home.join(".claude").join("skills").join(skill);
    fs::create_dir_all(&dir).is_ok() && fs::write(dir.join("SKILL.md"), body).is_ok()
}

fn write_codex(home: &Path, skill: &str, body: &[u8]) -> bool {
    let dir = home.join(".codex");
    fs::create_dir_all(&dir).is_ok() && inject_block(&dir.join("AGENTS.md"), skill, body).is_ok()
}

fn write_pi(dir: &Path, skill: &str, body: &[u8]) -> bool {
    fs::write(dir.join(format!("{skill}.md")), body).is_ok()
}
