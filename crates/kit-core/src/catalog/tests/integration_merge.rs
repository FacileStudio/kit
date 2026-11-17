use super::super::{merge, parse_list_source, Manifest, Tool};

fn manifest(tools: &[(&str, &str)]) -> Manifest {
    let tools = tools
        .iter()
        .map(|(name, repo)| Tool {
            name: name.to_string(),
            repo: repo.to_string(),
            ..Tool::default()
        })
        .collect();
    Manifest {
        tools,
        ..Manifest::default()
    }
}

#[test]
fn merge_layers() {
    let base = manifest(&[("A", "base/A"), ("B", "base/B"), ("C", "base/C")]);
    let merged = merge(
        &base,
        &[manifest(&[("B", "layer/B-shadow"), ("D", "layer/D")])],
    );
    assert_eq!(merged.names(), ["A", "B", "C", "D"]);
    assert_eq!(merged.tools[1].repo, "layer/B-shadow");
    assert_eq!(merged.tools[3].repo, "layer/D");
    assert_eq!(base.tools[1].repo, "base/B", "the base is not changed");

    let merged = merge(
        &manifest(&[("MyTool", "base/mytool"), ("Other", "base/other")]),
        &[manifest(&[("mytool", "layer/mytool")])],
    );
    assert_eq!(merged.tools.len(), 2);
    assert_eq!(merged.tools[0].repo, "layer/mytool");

    let layers = [
        manifest(&[("C", "")]),
        manifest(&[("B", "layer2/B"), ("D", "")]),
    ];
    let merged = merge(&manifest(&[("A", ""), ("B", "")]), &layers);
    assert_eq!(merged.names(), ["A", "B", "C", "D"]);
    assert_eq!(merged.tools[1].repo, "layer2/B");

    assert_eq!(merge(&manifest(&[("A", "")]), &[]).names(), ["A"]);
}

#[test]
fn list_source_parsing() {
    let cases: &[(&str, &[&str])] = &[
        ("# community tools\nowner/repo1\nowner/repo2\n\n# another section\nowner/repo3", &["owner/repo1", "owner/repo2", "owner/repo3"]),
        ("<!DOCTYPE html>\n<html><body>\n  <a href=\"/owner/repo1/\">repo1</a>\n  <a href=\"/owner/repo2/\">repo2</a>\n</body></html>", &["owner/repo1", "owner/repo2"]),
        ("owner/repo1\nowner/repo2\nowner/repo1\nowner/repo2\nowner/repo3", &["owner/repo1", "owner/repo2", "owner/repo3"]),
        ("", &[]),
        ("\n# comment\n  # indented comment\n\n", &[]),
        ("<a href=\"/a/b/\"></a>\n<a href=\"/c/d/\"></a>\n<a href=\"/a/b/\"></a>", &["a/b", "c/d"]),
        ("owner/repo\nnot-a-repo\nanother/valid/repo\nsingleword\nowner/ok", &["owner/repo", "owner/ok"]),
    ];
    for (body, want) in cases {
        assert_eq!(parse_list_source(body), *want, "body {body:?}");
    }
}
