use smithcraft::formats;
#[test]
fn read_original_distribution_data() {
    let dir = std::path::Path::new(r"C:\Program Files (x86)\Smith V4.1\S-parameter");
    if !dir.exists() {
        return;
    }
    let mut errors = vec![];
    let mut count = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let p = entry.unwrap().path();
        let ext = p
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if !["s1p", "s2p", "cti", "txt"].contains(&ext.as_str()) {
            continue;
        }
        match formats::read_network(&p) {
            Ok(n) => {
                assert!(!n.points.is_empty());
                count += 1;
            }
            Err(e) => errors.push(format!("{}: {e}", p.display())),
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    assert!(count >= 40, "read {count} files");
}
