//! `archcheck` v0 (XT-01): the real crate graph must equal the allowed edges of the
//! Authoritative Dependency Graph (`docs/production/module-boundaries.md` Part B), and a
//! use-level scan rejects forbidden imports and tokens.
//!
//! Only normal and build dependencies are evaluated; dev-dependencies are ignored
//! (rule R-DEVCYCLE of the ADG).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// One rule violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Stable rule id.
    pub rule: &'static str,
    /// Crate (or file) the violation is about.
    pub subject: String,
    /// Human readable detail.
    pub detail: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.rule, self.subject, self.detail)
    }
}

const BACKENDS: [&str; 3] = [
    "cube-backend-voxel",
    "cube-backend-sdf",
    "cube-backend-mesh",
];
const TOOLS: [&str; 3] = ["cube-tool-sculpt", "cube-tool-boolean", "cube-resolution"];
const BASE: [&str; 3] = ["cube-core", "cube-processing", "cube-editing-api"];

/// Allowed direct (`D`) workspace dependencies per crate, from ADG Part B. Edges marked `A`
/// (abstraction through an L2 port) are deliberately absent: they must not be direct.
pub fn allowed_edges() -> BTreeMap<&'static str, BTreeSet<&'static str>> {
    let mut m: BTreeMap<&'static str, BTreeSet<&'static str>> = BTreeMap::new();
    let mut put = |name: &'static str, deps: &[&'static str]| {
        m.insert(name, deps.iter().copied().collect());
    };
    put("cube-core", &[]);
    put("cube-processing", &["cube-core"]);
    put("cube-editing-api", &["cube-core", "cube-processing"]);
    put("cube-orchestration", &BASE);
    for b in BACKENDS {
        put(b, &BASE);
    }
    for t in TOOLS {
        put(t, &BASE);
    }
    put("cube-history", &BASE);
    put("cube-composite", &BASE);
    put("cube-persistence", &BASE);
    put("cube-worldgen", &["cube-core", "cube-processing"]);
    put("cube-net", &["cube-core", "cube-editing-api"]);
    let mut server: Vec<&'static str> = BASE.to_vec();
    server.extend(["cube-orchestration", "cube-history", "cube-composite"]);
    server.extend(["cube-persistence", "cube-worldgen", "cube-net"]);
    server.extend(BACKENDS);
    server.extend(TOOLS);
    put("cube-server", &server);
    let mut client: Vec<&'static str> = BASE.to_vec();
    client.extend(BACKENDS);
    client.push("cube-net");
    put("cube-client", &client);
    let mut adapter: Vec<&'static str> = BASE.to_vec();
    adapter.push("cube-client");
    // The adapter reaches the world only through the composition root (DEC-092).
    adapter.push("cube-app");
    put("cube-godot-adapter", &adapter);
    // Composition root of the first world (DEC-092): joins the local host and the client.
    put(
        "cube-app",
        &[
            "cube-core",
            "cube-processing",
            "cube-orchestration",
            "cube-persistence",
            "cube-backend-voxel",
            "cube-client",
            "cube-server",
        ],
    );
    put("cube-testkit", &BASE);
    put("xtask", &[]);
    let everything: Vec<&'static str> = m.keys().copied().collect();
    m.insert("cube-integration-tests", everything.into_iter().collect());
    m
}

/// Crates in which an async runtime must not be a dependency (decision D-E h).
fn lower_layer(name: &str) -> bool {
    !matches!(
        name,
        "cube-net"
            | "cube-server"
            | "cube-client"
            | "cube-godot-adapter"
            | "cube-app"
            | "cube-testkit"
            | "cube-integration-tests"
            | "xtask"
    )
}

const ASYNC_RUNTIMES: [&str; 3] = ["tokio", "async-std", "smol"];

/// Checks the workspace crate graph given `cargo metadata --no-deps` output.
pub fn check_graph(meta: &Value) -> Vec<Violation> {
    let allowed = allowed_edges();
    let mut out = Vec::new();
    let packages = meta["packages"].as_array().cloned().unwrap_or_default();
    let members: BTreeSet<String> = packages
        .iter()
        .filter_map(|p| p["name"].as_str().map(str::to_owned))
        .collect();
    for p in &packages {
        let Some(name) = p["name"].as_str() else {
            continue;
        };
        let Some(allowed_deps) = allowed.get(name) else {
            out.push(Violation {
                rule: "ARCH-UNKNOWN-CRATE",
                subject: name.to_owned(),
                detail:
                    "crate is not in the ADG module grid (add it to Part A/B and archcheck first)"
                        .into(),
            });
            continue;
        };
        for d in p["dependencies"].as_array().cloned().unwrap_or_default() {
            if d["kind"].as_str() == Some("dev") {
                continue;
            }
            // `loom` is allowed in cube-core only as a `cfg(loom)` target dependency (model checking).
            if name == "cube-core"
                && d["name"].as_str() == Some("loom")
                && d["target"].as_str() == Some("cfg(loom)")
            {
                continue;
            }
            let Some(dep) = d["name"].as_str() else {
                continue;
            };
            if members.contains(dep) {
                if !allowed_deps.contains(dep) {
                    out.push(Violation {
                        rule: "ARCH-EDGE",
                        subject: name.to_owned(),
                        detail: format!(
                            "dependency on `{dep}` is not an allowed edge of ADG Part B"
                        ),
                    });
                }
                continue;
            }
            if name == "cube-core" {
                out.push(Violation {
                    rule: "ARCH-CORE-DEPS",
                    subject: name.to_owned(),
                    detail: format!(
                        "cube-core must have zero external dependencies (D-E j); found `{dep}`"
                    ),
                });
            }
            if dep.starts_with("godot") && name != "cube-godot-adapter" {
                out.push(Violation {
                    rule: "ARCH-ENGINE-DEP",
                    subject: name.to_owned(),
                    detail: format!("engine crate `{dep}` is allowed only in cube-godot-adapter (AOD-04, AOD-06)"),
                });
            }
            if lower_layer(name) && ASYNC_RUNTIMES.contains(&dep) {
                out.push(Violation {
                    rule: "ARCH-ASYNC",
                    subject: name.to_owned(),
                    detail: format!(
                        "async runtime `{dep}` is not allowed below the server/client layer (D-E h)"
                    ),
                });
            }
        }
    }
    out
}

fn is_tool_crate(krate: &str) -> bool {
    matches!(krate, "xtask" | "cube-testkit" | "cube-integration-tests")
}

fn contains_word(line: &str, word: &str) -> bool {
    let bytes = line.as_bytes();
    let mut from = 0;
    while let Some(i) = line[from..].find(word) {
        let start = from + i;
        let end = start + word.len();
        let before_ok =
            start == 0 || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
        let after_ok =
            end >= bytes.len() || !(bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_');
        if before_ok && after_ok {
            return true;
        }
        from = end;
    }
    false
}

/// Mutation-boundary rules (XT-02, invariant MB-1): which crates may name which handle type.
const HANDLE_RULES: [(&str, &[&str]); 5] = [
    (
        "UserCommit",
        &[
            "cube-core",
            "cube-orchestration",
            "cube-server",
            "cube-integration-tests",
        ],
    ),
    (
        "SystemCommit",
        &[
            "cube-core",
            "cube-worldgen",
            "cube-server",
            "cube-integration-tests",
        ],
    ),
    (
        "RecoveryCommit",
        &[
            "cube-core",
            "cube-persistence",
            "cube-server",
            "cube-integration-tests",
        ],
    ),
    (
        "AssetHost",
        &["cube-core", "cube-server", "cube-integration-tests"],
    ),
    (
        "into_handles",
        &["cube-core", "cube-server", "cube-integration-tests"],
    ),
];

/// Crates that must hold no history or commit logic of their own.
fn is_adapter(krate: &str) -> bool {
    matches!(krate, "cube-client" | "cube-godot-adapter")
}

/// Use-level scan of one source file. A line may carry `// archcheck:allow(RULE) reason`.
pub fn scan_source(krate: &str, file: &str, text: &str) -> Vec<Violation> {
    let mut out = Vec::new();
    for (no, raw) in text.lines().enumerate() {
        let (code, comment) = match raw.find("//") {
            Some(i) => (&raw[..i], &raw[i..]),
            None => (raw, ""),
        };
        let allow = |rule: &str| comment.contains(&format!("archcheck:allow({rule})"));
        let mut report = |rule: &'static str, detail: &str| {
            if !allow(rule) {
                out.push(Violation {
                    rule,
                    subject: format!("{file}:{}", no + 1),
                    detail: detail.to_owned(),
                });
            }
        };
        if !is_tool_crate(krate) && (code.contains("std::process") || code.contains("Command::new"))
        {
            report(
                "ARCH-PROCESS",
                "child processes are forbidden in production crates (AOD-12)",
            );
        }
        if krate != "cube-godot-adapter" && !is_tool_crate(krate) && code.contains("godot::") {
            report(
                "ARCH-ENGINE-USE",
                "engine types are allowed only in cube-godot-adapter (AOD-04)",
            );
        }
        for (name, allowed) in HANDLE_RULES {
            if contains_word(code, name) && !allowed.contains(&krate) && !is_tool_crate(krate) {
                report(
                    "ARCH-HANDLE",
                    &format!("`{name}` may only be named by {allowed:?} (mutation boundary, MB-1)"),
                );
            }
        }
        if krate == "cube-orchestration"
            && contains_word(code, "UserCommit")
            && !file.ends_with("src/commit_path.rs")
        {
            report(
                "ARCH-USER-COMMIT-PATH",
                "only src/commit_path.rs of cube-orchestration may name UserCommit (single user commit path, ORCH-05)",
            );
        }
        if is_adapter(krate) {
            for pat in ["undo", "redo", "EditBatch", "commit"] {
                if contains_word(code, pat) {
                    report(
                        "ARCH-ADAPTER-LOGIC",
                        &format!(
                            "adapters and clients hold no history or commit logic (found `{pat}`)"
                        ),
                    );
                }
            }
        }
        if krate == "cube-core" {
            for pat in [
                "std::fs",
                "std::net",
                "std::env",
                "std::time",
                "SystemTime",
                "Instant",
                "std::thread::sleep",
                "rand::",
                "getrandom",
            ] {
                if code.contains(pat) {
                    report(
                        "ARCH-CORE-IO",
                        &format!(
                            "cube-core has no I/O, clock, environment or randomness (found `{pat}`)"
                        ),
                    );
                }
            }
        }
        if krate == "cube-core" && (contains_word(code, "f32") || contains_word(code, "f64")) {
            report(
                "ARCH-FLOAT",
                "no floating point in authoritative Core code (D-T)",
            );
        }
    }
    out
}

/// `true` if the text contains an unresolved merge conflict: a line that starts with the opening
/// marker followed later by a line that starts with the closing marker. Lines carrying
/// `archcheck:allow(ARCH-CONFLICT-MARKER)` are ignored so that fixtures may contain markers.
pub fn has_conflict_markers(text: &str) -> bool {
    let open = "<".repeat(7) + " ";
    let close = ">".repeat(7) + " ";
    let mut opened = false;
    for line in text.lines() {
        if line.contains("archcheck:allow(ARCH-CONFLICT-MARKER)") {
            continue;
        }
        if line.starts_with(&open) {
            opened = true;
        } else if opened && line.starts_with(&close) {
            return true;
        }
    }
    false
}

fn collect_text_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let path = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if !matches!(
                name.as_str(),
                "target" | ".git" | "node_modules" | "fixtures"
            ) {
                collect_text_files(&path, out);
            }
        } else if matches!(
            path.extension().and_then(|x| x.to_str()),
            Some("rs" | "md" | "toml" | "yml" | "yaml" | "json" | "py" | "txt" | "stderr")
        ) {
            out.push(path);
        }
    }
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Runs the full check on the workspace rooted at `root` (needs `cargo` on PATH).
pub fn run(root: &Path) -> Result<Vec<Violation>, String> {
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("cannot run cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let meta: Value =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("bad metadata json: {e}"))?;
    let mut violations = check_graph(&meta);
    for p in meta["packages"].as_array().cloned().unwrap_or_default() {
        let (Some(name), Some(manifest)) = (p["name"].as_str(), p["manifest_path"].as_str()) else {
            continue;
        };
        let Some(dir) = Path::new(manifest).parent() else {
            continue;
        };
        let mut files = Vec::new();
        collect_rs(&dir.join("src"), &mut files);
        files.sort();
        for f in files {
            let text = std::fs::read_to_string(&f).map_err(|e| format!("{}: {e}", f.display()))?;
            let rel = f
                .strip_prefix(root)
                .unwrap_or(&f)
                .display()
                .to_string()
                .replace('\\', "/");
            violations.extend(scan_source(name, &rel, &text));
        }
    }
    let mut text_files = Vec::new();
    collect_text_files(root, &mut text_files);
    text_files.sort();
    for f in text_files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        if has_conflict_markers(&text) {
            violations.push(Violation {
                rule: "ARCH-CONFLICT-MARKER",
                subject: f
                    .strip_prefix(root)
                    .unwrap_or(&f)
                    .display()
                    .to_string()
                    .replace('\\', "/"),
                detail: "unresolved merge conflict markers".to_owned(),
            });
        }
    }
    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cube_testkit::{DetRng, for_each_seed};
    use serde_json::json;

    fn pkg(name: &str, deps: &[(&str, Option<&str>)]) -> Value {
        let deps: Vec<Value> = deps
            .iter()
            .map(|(n, k)| json!({ "name": n, "kind": k }))
            .collect();
        json!({ "name": name, "dependencies": deps, "manifest_path": "x/Cargo.toml" })
    }
    fn meta(pkgs: Vec<Value>) -> Value {
        json!({ "packages": pkgs })
    }
    fn rules(v: &[Violation]) -> Vec<&'static str> {
        v.iter().map(|x| x.rule).collect()
    }

    #[test]
    fn clean_graph_passes() {
        let m = meta(vec![
            pkg("cube-core", &[]),
            pkg("cube-processing", &[("cube-core", None)]),
            pkg("cube-testkit", &[("cube-core", None)]),
        ]);
        assert!(check_graph(&m).is_empty());
    }

    // ---- seeded violations: every one must be caught ----

    #[test]
    fn seeded_core_depends_on_processing() {
        let m = meta(vec![
            pkg("cube-core", &[("cube-processing", None)]),
            pkg("cube-processing", &[]),
        ]);
        assert_eq!(rules(&check_graph(&m)), ["ARCH-EDGE"]);
    }

    #[test]
    fn seeded_core_external_dependency() {
        let m = meta(vec![pkg("cube-core", &[("rand", None)])]);
        assert_eq!(rules(&check_graph(&m)), ["ARCH-CORE-DEPS"]);
    }

    #[test]
    fn loom_is_allowed_in_core_only_as_a_cfg_loom_dependency() {
        let with_target = |t: Value| {
            let dep = json!({ "name": "loom", "kind": null, "target": t });
            json!({ "packages": [{ "name": "cube-core", "dependencies": [dep], "manifest_path": "x/Cargo.toml" }] })
        };
        assert!(check_graph(&with_target(json!("cfg(loom)"))).is_empty());
        assert_eq!(
            rules(&check_graph(&with_target(Value::Null))),
            ["ARCH-CORE-DEPS"]
        );
        assert_eq!(
            rules(&check_graph(&with_target(json!("cfg(unix)")))),
            ["ARCH-CORE-DEPS"]
        );
    }

    #[test]
    fn seeded_io_clock_and_randomness_in_core_are_caught() {
        for src in [
            "use std::fs::File;",
            "let t = std::time::Instant::now();",
            "let n = SystemTime::now();",
            "use rand::Rng;",
            "std::env::var(\"X\")",
        ] {
            let r = rules(&scan_source("cube-core", "src/lib.rs", src));
            assert!(
                !r.is_empty() && r.iter().all(|x| *x == "ARCH-CORE-IO"),
                "{src}"
            );
        }
        assert!(scan_source("cube-core", "src/lib.rs", "// std::fs in a comment").is_empty());
        assert!(scan_source("cube-persistence", "src/lib.rs", "use std::fs::File;").is_empty());
        assert!(scan_source("xtask", "src/main.rs", "use std::fs;").is_empty());
    }

    #[test]
    fn seeded_unknown_crate() {
        let m = meta(vec![pkg("cube-mystery", &[])]);
        assert_eq!(rules(&check_graph(&m)), ["ARCH-UNKNOWN-CRATE"]);
    }

    #[test]
    fn seeded_forbidden_l2_bypass() {
        // orchestration -> history is an `A` edge in the ADG: must go through a port, not directly.
        let m = meta(vec![
            pkg("cube-orchestration", &[("cube-history", None)]),
            pkg("cube-history", &[]),
        ]);
        assert_eq!(rules(&check_graph(&m)), ["ARCH-EDGE"]);
    }

    #[test]
    fn seeded_engine_and_async_dependencies() {
        let m = meta(vec![pkg(
            "cube-persistence",
            &[("godot", None), ("tokio", None)],
        )]);
        let mut r = rules(&check_graph(&m));
        r.sort_unstable();
        assert_eq!(r, ["ARCH-ASYNC", "ARCH-ENGINE-DEP"]);
    }

    #[test]
    fn build_dependency_counts_dev_does_not() {
        let m = meta(vec![
            pkg("cube-core", &[("cube-testkit", Some("dev"))]),
            pkg("cube-testkit", &[]),
            pkg("cube-processing", &[("cube-testkit", Some("build"))]),
        ]);
        assert_eq!(rules(&check_graph(&m)), ["ARCH-EDGE"]);
    }

    #[test]
    fn seeded_process_and_float_and_engine_use() {
        let src = "fn a() { let _ = std::process::Command::new(\"x\"); }\nlet y: f64 = 1.0;\nuse godot::prelude::*;\n";
        let r = rules(&scan_source("cube-core", "src/lib.rs", src));
        assert!(
            r.contains(&"ARCH-PROCESS")
                && r.contains(&"ARCH-FLOAT")
                && r.contains(&"ARCH-ENGINE-USE")
        );
    }

    #[test]
    fn scan_ignores_comments_words_and_allowed_lines() {
        let src = "// f64 in a comment\nlet f64x = 1; // identifier, not a type\nlet z: f32 = 0.0; // archcheck:allow(ARCH-FLOAT) render-only value\n";
        assert!(scan_source("cube-core", "src/lib.rs", src).is_empty());
        assert!(
            scan_source(
                "xtask",
                "src/main.rs",
                "std::process::Command::new(\"cargo\");"
            )
            .is_empty()
        );
        assert!(scan_source("cube-worldgen", "src/a.rs", "let v: f64 = 1.0;").is_empty());
    }

    #[test]
    fn seeded_worldgen_naming_the_user_handle_is_caught() {
        let src = "use cube_core::UserCommit;
fn f(h: &UserCommit) {}
";
        assert_eq!(
            rules(&scan_source("cube-worldgen", "src/lib.rs", src)),
            ["ARCH-HANDLE", "ARCH-HANDLE"]
        );
        assert!(
            scan_source(
                "cube-worldgen",
                "src/lib.rs",
                "use cube_core::SystemCommit;"
            )
            .is_empty()
        );
        assert_eq!(
            rules(&scan_source(
                "cube-persistence",
                "src/lib.rs",
                "SystemCommit"
            )),
            ["ARCH-HANDLE"]
        );
        assert_eq!(
            rules(&scan_source(
                "cube-client",
                "src/lib.rs",
                "let h = AssetHost::new(a);"
            )),
            ["ARCH-HANDLE"]
        );
        assert!(scan_source("cube-server", "src/lib.rs", "host.into_handles()").is_empty());
        assert!(scan_source("cube-orchestration", "src/commit_path.rs", "UserCommit").is_empty());
    }

    #[test]
    fn seeded_conflict_markers_are_caught_and_ordinary_text_is_not() {
        let open = "<".repeat(7);
        let close = ">".repeat(7);
        let bad = format!("a\n{open} HEAD\nmine\n=======\ntheirs\n{close} origin/dev\nb\n");
        assert!(has_conflict_markers(&bad));
        let fixture = format!(
            "{open} HEAD // archcheck:allow(ARCH-CONFLICT-MARKER)\n{close} x // archcheck:allow(ARCH-CONFLICT-MARKER)\n"
        );
        assert!(!has_conflict_markers(&fixture));
        // A setext heading underline and an indented marker inside code are not conflicts.
        assert!(!has_conflict_markers("Title\n=======\ntext\n"));
        assert!(!has_conflict_markers(&format!(
            "    let x = \"{open} \";\n"
        )));
        assert!(!has_conflict_markers(&format!("{open} only an opener\n")));
    }

    #[test]
    fn seeded_user_commit_outside_the_single_commit_path_is_caught() {
        let src = "fn f(h: &UserCommit) {}";
        assert_eq!(
            rules(&scan_source(
                "cube-orchestration",
                "crates/cube-orchestration/src/txn.rs",
                src
            )),
            ["ARCH-USER-COMMIT-PATH"]
        );
        assert!(
            scan_source(
                "cube-orchestration",
                "crates/cube-orchestration/src/commit_path.rs",
                src
            )
            .is_empty()
        );
    }

    #[test]
    fn seeded_history_or_commit_logic_in_adapters_is_caught() {
        for pat in [
            "fn undo() {}",
            "let b: EditBatch;",
            "world.commit(op, batch);",
            "redo()",
        ] {
            assert_eq!(
                rules(&scan_source("cube-godot-adapter", "src/lib.rs", pat)),
                ["ARCH-ADAPTER-LOGIC"],
                "{pat}"
            );
        }
        assert!(
            scan_source(
                "cube-godot-adapter",
                "src/lib.rs",
                "// undo is not here
let recommitted = 1;"
            )
            .is_empty()
        );
        assert!(scan_source("cube-history", "src/lib.rs", "fn undo() {}").is_empty());
    }

    // ---- randomised: the checker must agree with an independent oracle on random graphs ----

    #[test]
    fn random_graphs_match_oracle() {
        let allowed = allowed_edges();
        let names: Vec<&str> = allowed.keys().copied().collect();
        for_each_seed(0xA2C4, 200, |seed| {
            let mut rng = DetRng::new(seed);
            let mut pkgs = Vec::new();
            let mut expected_bad = 0usize;
            for &n in &names {
                let mut deps: Vec<(&str, Option<&str>)> = Vec::new();
                for &d in &names {
                    if d != n && rng.below(12) == 0 {
                        let dev = rng.below(4) == 0;
                        deps.push((d, if dev { Some("dev") } else { None }));
                        if !dev && !allowed[n].contains(d) {
                            expected_bad += 1;
                        }
                    }
                }
                pkgs.push(pkg(n, &deps));
            }
            let got = check_graph(&meta(pkgs));
            assert_eq!(got.len(), expected_bad, "seed {seed:#x}: {got:?}");
            assert!(got.iter().all(|v| v.rule == "ARCH-EDGE"));
        });
    }

    #[test]
    fn module_grid_has_no_dependency_cycle() {
        // The allowed grid itself must be acyclic (Kahn's algorithm).
        let allowed = allowed_edges();
        let mut indeg: BTreeMap<&str, usize> = allowed.keys().map(|k| (*k, 0)).collect();
        for deps in allowed.values() {
            for _ in deps {}
        }
        for (k, deps) in &allowed {
            *indeg.get_mut(k).expect("known") = deps.len();
        }
        let mut ready: Vec<&str> = indeg
            .iter()
            .filter(|(_, d)| **d == 0)
            .map(|(k, _)| *k)
            .collect();
        let mut seen = 0;
        while let Some(n) = ready.pop() {
            seen += 1;
            for (k, deps) in &allowed {
                if deps.contains(n) {
                    let e = indeg.get_mut(k).expect("known");
                    *e -= 1;
                    if *e == 0 {
                        ready.push(k);
                    }
                }
            }
        }
        assert_eq!(seen, allowed.len(), "cycle in the allowed dependency grid");
    }
}
