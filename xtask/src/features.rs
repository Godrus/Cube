//! Feature registry check and generator. `docs/features.json` is the source; the Markdown
//! overview `docs/FEATURE-REGISTRY.md` is generated from it. `cargo xtask features --check`
//! validates the registry and that the Markdown is up to date; `--write` regenerates it.
//!
//! The checks are deliberately light: unique stable ids, known status words, required fields,
//! well-formed gate references, known dependencies, and statuses that contradict each other.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

const DECISION: [&str; 5] = ["IDEA", "PROPOSED", "APPROVED", "DEFERRED", "REJECTED"];
const IMPLEMENTATION: [&str; 5] = [
    "NOT_STARTED",
    "IN_PROGRESS",
    "IMPLEMENTED",
    "TESTED",
    "INTEGRATED",
];
const AVAILABILITY: [&str; 4] = [
    "NOT_AVAILABLE",
    "INTERNAL_ONLY",
    "DEMO_AVAILABLE",
    "PLAYER_AVAILABLE",
];
const PRIORITY: [&str; 4] = ["P0", "P1", "P2", "P3"];

fn rank(list: &[&str], v: &str) -> Option<usize> {
    list.iter().position(|x| *x == v)
}

fn text<'a>(f: &'a Value, k: &str) -> Option<&'a str> {
    f.get(k)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
}

fn strings(f: &Value, k: &str) -> Option<Vec<String>> {
    f.get(k)?
        .as_array()?
        .iter()
        .map(|x| x.as_str().map(str::to_owned))
        .collect()
}

fn valid_id(id: &str) -> bool {
    // F-<CATEGORY LETTERS>-<3 digits>
    let p: Vec<&str> = id.split('-').collect();
    p.len() == 3
        && p[0] == "F"
        && (2..=4).contains(&p[1].len())
        && p[1].chars().all(|c| c.is_ascii_uppercase())
        && p[2].len() == 3
        && p[2].chars().all(|c| c.is_ascii_digit())
}

fn valid_gate(g: &str) -> bool {
    let num = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
    g.strip_prefix("PS-")
        .or_else(|| g.strip_prefix("RG-"))
        .or_else(|| g.strip_prefix("DG-"))
        .or_else(|| g.strip_prefix('G'))
        .is_some_and(num)
}

fn valid_revision(v: &str) -> bool {
    (7..=40).contains(&v.len()) && v.chars().all(|c| c.is_ascii_hexdigit())
}

/// All problems found in the registry document.
pub fn validate(doc: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if doc.get("schema").and_then(Value::as_u64) != Some(1) {
        out.push("schema must be 1".to_owned());
    }
    let Some(features) = doc.get("features").and_then(Value::as_array) else {
        out.push("features must be an array".to_owned());
        return out;
    };
    let ids: BTreeSet<&str> = features
        .iter()
        .filter_map(|f| f.get("id").and_then(Value::as_str))
        .collect();
    let mut seen = BTreeSet::new();
    for f in features {
        let id = f.get("id").and_then(Value::as_str).unwrap_or("<no id>");
        let mut bad = |m: &str| out.push(format!("{id}: {m}"));
        if !valid_id(id) {
            bad("id must look like F-ABC-001");
        } else if !seen.insert(id.to_owned()) {
            bad("duplicate id");
        }
        for k in ["name", "description", "category", "owner", "criteria"] {
            if text(f, k).is_none() {
                bad(&format!("missing {k}"));
            }
        }
        let dec = text(f, "decision").unwrap_or("");
        let imp = text(f, "implementation").unwrap_or("");
        let av = text(f, "availability").unwrap_or("");
        let pr = text(f, "priority").unwrap_or("");
        if rank(&DECISION, dec).is_none() {
            bad(&format!("unknown decision {dec:?}"));
        }
        if rank(&IMPLEMENTATION, imp).is_none() {
            bad(&format!("unknown implementation {imp:?}"));
        }
        if rank(&AVAILABILITY, av).is_none() {
            bad(&format!("unknown availability {av:?}"));
        }
        if rank(&PRIORITY, pr).is_none() {
            bad(&format!("unknown priority {pr:?}"));
        }
        match text(f, "verified") {
            Some(v) if valid_revision(v) => {}
            _ => bad("verified must be a git revision"),
        }
        let evidence = strings(f, "evidence");
        let depends = strings(f, "depends");
        let gates = strings(f, "gates");
        let blockers = strings(f, "blockers");
        let blocked = f.get("blocked").and_then(Value::as_bool);
        if evidence.is_none()
            || depends.is_none()
            || gates.is_none()
            || blockers.is_none()
            || blocked.is_none()
        {
            bad(
                "evidence, depends, gates, blockers (arrays of text) and blocked (bool) are required",
            );
            continue;
        }
        let (evidence, depends, gates, blockers, blocked) = (
            evidence.unwrap_or_default(),
            depends.unwrap_or_default(),
            gates.unwrap_or_default(),
            blockers.unwrap_or_default(),
            blocked.unwrap_or(false),
        );
        for d in &depends {
            if !ids.contains(d.as_str()) {
                bad(&format!("depends on unknown feature {d}"));
            }
            if d == id {
                bad("depends on itself");
            }
        }
        for g in &gates {
            if !valid_gate(g) {
                bad(&format!("malformed gate reference {g:?}"));
            }
        }
        // Contradictions.
        let (di, ii, ai) = (
            rank(&DECISION, dec),
            rank(&IMPLEMENTATION, imp),
            rank(&AVAILABILITY, av),
        );
        if let Some(ii) = ii {
            if ii >= 2 && evidence.is_empty() {
                bad("IMPLEMENTED or beyond needs evidence");
            }
            if ii >= 1 && dec != "APPROVED" {
                bad("work started or finished on a feature that is not APPROVED");
            }
            if let Some(ai) = ai {
                if ai >= 1 && ii < 2 {
                    bad("available to someone but not implemented");
                }
                if ai >= 2 && ii < 4 {
                    bad("DEMO or PLAYER availability needs INTEGRATED");
                }
            }
        }
        if blocked && blockers.is_empty() {
            bad("blocked without blocker ids");
        }
        if !blocked && !blockers.is_empty() {
            bad("blocker ids on a feature that is not blocked");
        }
        if matches!(dec, "REJECTED" | "DEFERRED") && ii.is_some_and(|i| i >= 1) {
            bad("rejected or deferred but work exists");
        }
        let _ = di;
    }
    out
}

fn cell(s: &str) -> String {
    s.replace('|', "/").replace('\n', " ")
}

/// The generated Markdown overview.
pub fn render(doc: &Value) -> String {
    let features = doc["features"].as_array().cloned().unwrap_or_default();
    let mut by_cat: BTreeMap<usize, (String, Vec<&Value>)> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for f in &features {
        let c = f["category"].as_str().unwrap_or("").to_owned();
        if !order.contains(&c) {
            order.push(c.clone());
        }
        let i = order.iter().position(|x| *x == c).unwrap_or(0);
        by_cat.entry(i).or_insert_with(|| (c, Vec::new())).1.push(f);
    }
    let count = |k: &str, v: &str| features.iter().filter(|f| f[k].as_str() == Some(v)).count();
    let mut s = String::new();
    s.push_str("# Cube feature registry\n\n");
    s.push_str("> GENERATED from `docs/features.json` by `cargo xtask features --write`. Do not edit by hand; CI fails if this file is out of date.\n\n");
    s.push_str("Three separate dimensions: **Decision** (IDEA, PROPOSED, APPROVED, DEFERRED, REJECTED), **Implementation** (NOT_STARTED, IN_PROGRESS, IMPLEMENTED, TESTED, INTEGRATED) and **Player availability** (NOT_AVAILABLE, INTERNAL_ONLY, DEMO_AVAILABLE, PLAYER_AVAILABLE). *Blocked* is a flag with blocker ids, not a status. Integrated code is not a player feature. Ids are stable and never reused.\n\n");
    s.push_str(&format!(
        "**{} features.** Integrated: {}. Player available: {}. Approved: {}. Proposed: {}. Blocked: {}.\n\n",
        features.len(),
        count("implementation", "INTEGRATED"),
        count("availability", "PLAYER_AVAILABLE"),
        count("decision", "APPROVED"),
        count("decision", "PROPOSED"),
        features.iter().filter(|f| f["blocked"].as_bool() == Some(true)).count(),
    ));
    for (cat, list) in by_cat.values() {
        s.push_str(&format!("## {cat}\n\n"));
        s.push_str("| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |\n|---|---|---|---|---|---|---|---|---|---|---|\n");
        for f in list {
            let j = |k: &str| {
                f[k].as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join("; ")
                    })
                    .unwrap_or_default()
            };
            s.push_str(&format!(
                "| {} | **{}**: {} Acceptance: {} | {} | {} | {} | {} | {} | {} | {} | {} | `{}` |\n",
                cell(f["id"].as_str().unwrap_or("")),
                cell(f["name"].as_str().unwrap_or("")),
                cell(f["description"].as_str().unwrap_or("")),
                cell(f["criteria"].as_str().unwrap_or("")),
                f["decision"].as_str().unwrap_or(""),
                f["implementation"].as_str().unwrap_or(""),
                f["availability"].as_str().unwrap_or(""),
                f["priority"].as_str().unwrap_or(""),
                cell(&j("depends")),
                cell(&j("gates")),
                cell(&j("evidence")),
                if f["blocked"].as_bool() == Some(true) { cell(&j("blockers")) } else { "-".to_owned() },
                f["verified"].as_str().unwrap_or(""),
            ));
        }
        s.push('\n');
    }
    s
}

/// Runs the check (`write = false`) or regenerates the overview (`write = true`).
pub fn run(root: &Path, write: bool) -> Result<Vec<String>, String> {
    let src = root.join("docs/features.json");
    let md = root.join("docs/FEATURE-REGISTRY.md");
    let text = std::fs::read_to_string(&src).map_err(|e| format!("{}: {e}", src.display()))?;
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("features.json: {e}"))?;
    let problems = validate(&doc);
    if !problems.is_empty() {
        return Ok(problems);
    }
    let want = render(&doc);
    if write {
        std::fs::write(&md, want).map_err(|e| format!("{}: {e}", md.display()))?;
        return Ok(vec![]);
    }
    let have = std::fs::read_to_string(&md)
        .unwrap_or_default()
        .replace("\r\n", "\n");
    if have != want {
        return Ok(vec![
            "docs/FEATURE-REGISTRY.md is out of date: run `cargo xtask features --write`"
                .to_owned(),
        ]);
    }
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn feature(id: &str) -> Value {
        json!({"id":id,"name":"n","description":"d","category":"c","decision":"APPROVED","implementation":"INTEGRATED",
        "availability":"INTERNAL_ONLY","priority":"P0","depends":[],"gates":["PS-1","G1"],"owner":"o","criteria":"c",
        "evidence":["e"],"blocked":false,"blockers":[],"verified":"abcdef1"})
    }

    fn doc(fs: Vec<Value>) -> Value {
        json!({"schema":1,"features":fs})
    }

    fn with(mut f: Value, k: &str, v: Value) -> Value {
        f[k] = v;
        f
    }

    #[test]
    fn a_clean_registry_passes() {
        assert!(validate(&doc(vec![feature("F-AAA-001"), feature("F-AAA-002")])).is_empty());
    }

    #[test]
    fn seeded_violations_are_each_caught() {
        let cases: Vec<(Value, &str)> = vec![
            (
                with(feature("F-AAA-001"), "id", json!("X-1")),
                "id must look like",
            ),
            (
                with(feature("F-AAA-001"), "decision", json!("MAYBE")),
                "unknown decision",
            ),
            (
                with(feature("F-AAA-001"), "implementation", json!("DONE")),
                "unknown implementation",
            ),
            (
                with(feature("F-AAA-001"), "availability", json!("SOON")),
                "unknown availability",
            ),
            (
                with(feature("F-AAA-001"), "name", json!("")),
                "missing name",
            ),
            (
                with(feature("F-AAA-001"), "gates", json!(["PS-x"])),
                "malformed gate",
            ),
            (
                with(feature("F-AAA-001"), "depends", json!(["F-ZZZ-999"])),
                "unknown feature",
            ),
            (
                with(feature("F-AAA-001"), "evidence", json!([])),
                "needs evidence",
            ),
            (
                with(feature("F-AAA-001"), "decision", json!("PROPOSED")),
                "not APPROVED",
            ),
            (
                with(
                    with(feature("F-AAA-001"), "implementation", json!("IMPLEMENTED")),
                    "availability",
                    json!("PLAYER_AVAILABLE"),
                ),
                "needs INTEGRATED",
            ),
            (
                with(
                    with(feature("F-AAA-001"), "implementation", json!("NOT_STARTED")),
                    "availability",
                    json!("INTERNAL_ONLY"),
                ),
                "not implemented",
            ),
            (
                with(feature("F-AAA-001"), "blocked", json!(true)),
                "without blocker ids",
            ),
            (
                with(feature("F-AAA-001"), "blockers", json!(["GOV-1"])),
                "not blocked",
            ),
            (
                with(feature("F-AAA-001"), "verified", json!("main")),
                "git revision",
            ),
            (
                with(feature("F-AAA-001"), "decision", json!("DEFERRED")),
                "deferred but work exists",
            ),
        ];
        for (f, want) in cases {
            let problems = validate(&doc(vec![f]));
            assert!(
                problems.iter().any(|p| p.contains(want)),
                "{want}: {problems:?}"
            );
        }
        let dup = validate(&doc(vec![feature("F-AAA-001"), feature("F-AAA-001")]));
        assert!(dup.iter().any(|p| p.contains("duplicate")));
    }

    #[test]
    fn the_overview_is_deterministic_and_counts_features() {
        let d = doc(vec![
            feature("F-AAA-001"),
            with(feature("F-BBB-001"), "category", json!("Other")),
        ]);
        let a = render(&d);
        assert_eq!(a, render(&d));
        assert!(a.contains("**2 features.**") && a.contains("## c") && a.contains("## Other"));
    }
}
