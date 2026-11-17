use clap::CommandFactory;

use super::list::Row;
use super::list_render::state_of;
use super::names::{self_error, split_names, split_self};
use crate::cli::Cli;

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn split_names_splits_commas_and_whitespace() {
    let cases: [(&[&str], &[&str]); 9] = [
        (&[], &[]),
        (&["poke"], &["poke"]),
        (&["poke", "palette"], &["poke", "palette"]),
        (&["poke,palette"], &["poke", "palette"]),
        (&["poke, palette"], &["poke", "palette"]),
        (
            &["poke,palette", "widgets"],
            &["poke", "palette", "widgets"],
        ),
        (&["poke palette"], &["poke", "palette"]),
        (&[",poke,", ",palette,"], &["poke", "palette"]),
        (&["poke,Poke", "poke"], &["poke"]),
    ];
    for (input, want) in cases {
        assert_eq!(split_names(&strings(input)), strings(want), "{input:?}");
    }
}

#[test]
fn split_self_takes_kit_out_of_an_update() {
    assert_eq!(
        split_self(&[]),
        (true, vec![], false),
        "no arguments takes kit and everything listed"
    );
    assert_eq!(
        split_self(&strings(&["kit"])),
        (true, vec![], true),
        "kit alone updates kit alone"
    );
    assert_eq!(
        split_self(&strings(&["nuage"])),
        (false, strings(&["nuage"]), true),
        "a named tool leaves kit out"
    );
    let (me, rest, _) = split_self(&strings(&["nuage", "kit"]));
    assert!(
        me && rest == strings(&["nuage"]),
        "kit alongside a tool takes both"
    );
}

#[test]
fn unknown_tool_answers_for_kit() {
    let message = self_error().to_string();
    assert!(message.contains("kit self-update"), "{message}");
    assert!(
        !message.contains("unknown"),
        "kit is a listing row; calling it unknown reads as a bug"
    );
}

#[test]
fn root_version_is_inherited_by_every_subcommand() {
    let root = Cli::command();
    assert_eq!(root.get_version(), Some(super::VERSION));
    let rendered = root.clone().render_version();
    assert_eq!(rendered.trim(), format!("kit {}", super::VERSION));
    for sub in root.get_subcommands() {
        assert!(
            sub.get_about().is_some(),
            "{} has no help line",
            sub.get_name()
        );
    }
}

#[test]
fn state_of_reads_like_the_listing() {
    let row = |installed, version: &str, latest: &str, outdated| Row {
        installed,
        version: version.into(),
        latest: latest.into(),
        outdated,
        ..Row::default()
    };
    assert_eq!(
        state_of(&row(true, "1.2.3", "1.3.0", true)),
        "1.2.3 → 1.3.0"
    );
    assert_eq!(state_of(&row(true, "1.2.3", "1.2.3", false)), "1.2.3");
    assert_eq!(state_of(&row(false, "", "", false)), "not installed");
    assert_eq!(state_of(&row(true, "1.2.3", "", false)), "1.2.3");
    assert_eq!(state_of(&row(true, "", "", false)), "installed");
}

#[test]
fn help_lines_follow_the_wording_rules() {
    let root = Cli::command();
    for sub in root.get_subcommands() {
        let about = sub.get_about().map(|a| a.to_string()).unwrap_or_default();
        assert!(
            about.chars().next().is_some_and(char::is_uppercase),
            "{about}"
        );
        assert!(!about.ends_with('.'), "{about}");
    }
    root.debug_assert();
}
