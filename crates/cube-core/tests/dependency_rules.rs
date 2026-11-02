//! Spike-level dependency-direction check (a minimal form of rule DEP-01 for this workspace).
//!
//! Reads the Cargo manifests of all workspace crates and enforces:
//! - only `cube-godot-adapter` may depend on Godot;
//! - the allowed internal dependency edges (one direction, no cycles);
//! - no alternative-engine crates anywhere;
//! - `cube-core` has no dependencies at all.
//!
//! This does not close blocker B-9; it is an experimental check on the spike branch only.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

const ALLOWED: &[(&str, &[&str])] = &[
    ("cube-core", &[]),
    ("cube-processing", &["cube-core"]),
    ("cube-editing", &["cube-core", "cube-processing"]),
    ("cube-persistence", &["cube-core"]),
    (
        "cube-server",
        &[
            "cube-core",
            "cube-processing",
            "cube-editing",
            "cube-persistence",
        ],
    ),
    (
        "cube-godot-adapter",
        &[
            "cube-core",
            "cube-processing",
            "cube-editing",
            "cube-persistence",
            "godot",
        ],
    ),
];

const FORBIDDEN_EVERYWHERE: &[&str] = &[
    "unity",
    "bevy",
    "fyrox",
    "ue4",
    "unreal",
    "csharp",
    "cxx",
    "godot-csharp",
];

fn dependencies(manifest: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_deps = false;
    for line in manifest.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            in_deps =
                l == "[dependencies]" || l == "[dev-dependencies]" || l == "[build-dependencies]";
            continue;
        }
        if in_deps && !l.is_empty() && !l.starts_with('#') {
            if let Some((name, _)) = l.split_once('=') {
                deps.push(name.trim().to_string());
            }
        }
    }
    deps
}

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn manifests_follow_allowed_edges() {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for (name, allowed) in ALLOWED {
        let path = crates_dir().join(name).join("Cargo.toml");
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let deps = dependencies(&text);
        for d in &deps {
            assert!(
                allowed.contains(&d.as_str()),
                "{name} must not depend on {d}"
            );
            assert!(
                !FORBIDDEN_EVERYWHERE.contains(&d.as_str()),
                "{name}: forbidden engine/stack crate {d}"
            );
        }
        graph.insert(name.to_string(), deps);
    }
    assert!(
        graph["cube-core"].is_empty(),
        "cube-core must have no dependencies"
    );
    for (name, deps) in &graph {
        if name != "cube-godot-adapter" {
            assert!(
                !deps.iter().any(|d| d.contains("godot")),
                "{name} depends on Godot"
            );
        }
    }
}

#[test]
fn internal_graph_has_no_cycle() {
    let mut g: HashMap<&str, Vec<&str>> = HashMap::new();
    for (name, allowed) in ALLOWED {
        g.insert(
            name,
            allowed
                .iter()
                .copied()
                .filter(|d| d.starts_with("cube-"))
                .collect(),
        );
    }
    fn visit<'a>(
        n: &'a str,
        g: &HashMap<&'a str, Vec<&'a str>>,
        stack: &mut HashSet<&'a str>,
        done: &mut HashSet<&'a str>,
    ) {
        assert!(!stack.contains(n), "cycle through {n}");
        if done.contains(n) {
            return;
        }
        stack.insert(n);
        for m in &g[n] {
            visit(m, g, stack, done);
        }
        stack.remove(n);
        done.insert(n);
    }
    let (mut stack, mut done) = (HashSet::new(), HashSet::new());
    for n in g.keys() {
        visit(n, &g, &mut stack, &mut done);
    }
}

#[test]
fn core_sources_mention_no_engine_types() {
    fn scan(dir: &std::path::Path, hits: &mut Vec<String>) {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                scan(&p, hits);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let text = fs::read_to_string(&p).unwrap();
                for (i, line) in text.lines().enumerate() {
                    let code = line.split("//").next().unwrap_or("");
                    for needle in [
                        "godot::",
                        "use godot",
                        "SceneTree",
                        "GDScript",
                        ".tscn",
                        ".tres",
                    ] {
                        if code.contains(needle) {
                            hits.push(format!("{}:{}: {needle}", p.display(), i + 1));
                        }
                    }
                }
            }
        }
    }
    let mut hits = Vec::new();
    for c in [
        "cube-core",
        "cube-processing",
        "cube-editing",
        "cube-persistence",
        "cube-server",
    ] {
        scan(&crates_dir().join(c).join("src"), &mut hits);
    }
    assert!(
        hits.is_empty(),
        "engine references in non-adapter crates: {hits:?}"
    );
}
