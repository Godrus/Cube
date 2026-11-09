//! Architecture guard (AOD-04, DEC-066, DEC-068 j): Core has no dependencies, no engine types and
//! no content names. `cargo xtask archcheck` checks the crate graph; this test checks the sources.

use std::fs;
use std::path::Path;

fn sources(dir: &Path, out: &mut Vec<String>) {
    for e in fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            sources(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(fs::read_to_string(&p).unwrap());
        }
    }
}

#[test]
fn manifest_has_no_normal_dependencies() {
    let m = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    assert!(
        !m.lines().any(|l| l.trim() == "[dependencies]"),
        "cube-core must have zero dependencies"
    );
    assert!(!m.to_lowercase().contains("godot"));
}

#[test]
fn sources_mention_no_engine_or_content() {
    let mut all = Vec::new();
    sources(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut all);
    let text = all.join("\n").to_lowercase();
    for forbidden in ["godot", "namespace:", "cube:stone", "gdscript"] {
        assert!(
            !text.contains(forbidden),
            "forbidden word in Core sources: {forbidden}"
        );
    }
}
