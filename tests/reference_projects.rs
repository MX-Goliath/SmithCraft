//! Optional regression checks against the author's public example files.
//! The external files stay outside the repository; run after populating .reference/examples.
use smithcraft::{formats, rf::c};
#[test]
fn original_smith_project_endpoints() {
    let dir = std::path::Path::new(".reference/examples");
    if !dir.exists() {
        return;
    }
    let mut count = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "xmlsc") {
            continue;
        }
        let project =
            formats::load_project(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let text = std::fs::read_to_string(&path).unwrap();
        let doc = roxmltree::Document::parse(&text).unwrap();
        let expected: Vec<_> = doc
            .root_element()
            .children()
            .filter(|n| {
                n.has_tag_name("CDatapoint")
                    && n.children()
                        .any(|a| a.has_tag_name("IsRealDP") && a.text() == Some("0"))
            })
            .collect();
        let stages = project.stages();
        assert_eq!(stages.len() - 1, expected.len(), "{}", path.display());
        for (i, node) in expected.iter().enumerate() {
            let get = |name| {
                node.children()
                    .find(|n| n.has_tag_name(name))
                    .unwrap()
                    .text()
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
            };
            let z = c(get("R"), get("X"));
            let error = (z - stages[i + 1]).norm();
            assert!(
                error < 2e-4 * (1. + z.norm()),
                "{} stage {}: expected {z}, got {}, error {error}",
                path.display(),
                i + 1,
                stages[i + 1]
            );
        }
        count += 1;
    }
    assert_eq!(count, 22);
}
