//! Documentation check. `.agent/INDEX.md` is the allow-list of Markdown files; this check
//! fails when a tracked `.md` file is not covered by it, when a path listed in it does not
//! exist, when a relative Markdown link is broken, or when the short entry files grow.
//!
//! Every Markdown file needs at least one incoming link (the graph must stay connected), and a
//! backticked mention of another tracked `.md` file must be a link, not plain code text.
//!
//! A backticked token in the index counts as a path when it has no spaces. A token ending in
//! `/` covers a whole folder, `prefix/**` covers everything under `prefix/`, anything else
//! covers exactly that file.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

/// Lines allowed in the entry file.
const MAX_AGENTS_LINES: usize = 60;
/// Lines allowed in the short work log.
const MAX_LOG_LINES: usize = 40;
/// Listed paths that may be absent (not tracked or created on demand).
const MAY_BE_MISSING: [&str; 2] = [".agent/live/", ".agent/archive/"];

/// Backticked tokens of a Markdown text (single-line spans only).
fn backticked(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let mut parts = line.split('`');
        parts.next();
        while let (Some(inside), Some(_outside)) = (parts.next(), parts.next()) {
            out.push(inside.to_string());
        }
    }
    out
}

/// Path-like tokens of the index: no spaces, not empty.
fn index_tokens(index: &str) -> Vec<String> {
    backticked(index)
        .into_iter()
        .filter(|t| !t.is_empty() && !t.contains(char::is_whitespace))
        .collect()
}

/// True when the index token covers the tracked file.
fn covers(token: &str, file: &str) -> bool {
    if let Some(prefix) = token.strip_suffix("**") {
        file.starts_with(prefix)
    } else if token.ends_with('/') {
        file.starts_with(token)
    } else {
        token == file
    }
}

/// Normalises `a/b/../c` without touching the file system.
fn normalise(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Relative link targets (`](target)`) of a Markdown text, without fragments, skipping URLs.
fn links(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find("](") {
        rest = &rest[i + 2..];
        let Some(j) = rest.find(')') else { break };
        let target = &rest[..j];
        rest = &rest[j..];
        let path = target.split('#').next().unwrap_or("");
        let is_url = path
            .split_once(':')
            .is_some_and(|(s, _)| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphabetic()));
        if !path.is_empty() && !path.contains(char::is_whitespace) && !is_url {
            out.push(path.to_string());
        }
    }
    out
}

fn tracked_markdown(root: &Path) -> Result<Vec<String>, String> {
    let out = Command::new("git")
        .args(["ls-files", "*.md"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err("git ls-files failed".into());
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

fn line_count(root: &Path, rel: &str) -> usize {
    std::fs::read_to_string(root.join(rel)).map_or(0, |t| t.lines().count())
}

/// Files that are generated and therefore not linted for plain mentions.
const GENERATED: [&str; 1] = ["docs/FEATURE-REGISTRY.md"];
/// Files that may have no incoming link (imported by the harness, not linked).
const NO_INBOUND_NEEDED: [&str; 1] = ["CLAUDE.md"];

/// Removes inline Markdown links `[text](target)` from a line (their text and target).
fn without_links(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(i) = rest.find('[') {
        let after = &rest[i..];
        let end = after
            .find("](")
            .and_then(|j| after[j + 2..].find(')').map(|k| j + 2 + k));
        match end {
            Some(end) => {
                out.push_str(&rest[..i]);
                rest = &after[end + 1..];
            }
            None => {
                out.push_str(&rest[..=i]);
                rest = &rest[i + 1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Tracked file that a backticked token names, if any (the token must be a `.md` path).
fn resolves_to(token: &str, from: &str, tracked: &BTreeSet<String>) -> Option<String> {
    if !token.ends_with(".md") || token.contains(char::is_whitespace) {
        return None;
    }
    let dir = Path::new(from).parent().unwrap_or(Path::new(""));
    let mut cands = vec![normalise(&dir.join(token))];
    if token.contains('/') {
        cands.push(PathBuf::from(token));
        cands.push(PathBuf::from("docs").join(token));
    }
    cands
        .into_iter()
        .map(|c| c.to_string_lossy().replace('\\', "/"))
        .find(|c| c != from && tracked.contains(c))
}

/// Backticked spans of a line, outside links.
fn plain_code_spans(line: &str) -> Vec<String> {
    backticked(&without_links(line))
}

/// Runs the check; an empty list means everything is in order.
pub fn run(root: &Path) -> Result<Vec<String>, String> {
    let index_path = root.join(".agent/INDEX.md");
    let index = std::fs::read_to_string(&index_path)
        .map_err(|e| format!("{}: {e}", index_path.display()))?;
    let tokens = index_tokens(&index);
    let files = tracked_markdown(root)?;
    let mut bad = Vec::new();

    for f in &files {
        if !tokens.iter().any(|t| covers(t, f)) {
            bad.push(format!("not in .agent/INDEX.md: {f}"));
        }
    }

    for t in tokens.iter().filter(|t| t.contains('/')) {
        if MAY_BE_MISSING.iter().any(|m| t.starts_with(m)) {
            continue;
        }
        let rel = t.trim_end_matches('*').trim_end_matches('/');
        if !root.join(rel).exists() {
            bad.push(format!(".agent/INDEX.md lists a missing path: {t}"));
        }
    }

    let mut seen = BTreeSet::new();
    for f in &files {
        let Ok(text) = std::fs::read_to_string(root.join(f)) else {
            continue;
        };
        let dir = Path::new(f).parent().unwrap_or(Path::new(""));
        for target in links(&text) {
            let resolved = normalise(&dir.join(&target));
            if !root.join(&resolved).exists() && seen.insert((f.clone(), target.clone())) {
                bad.push(format!("broken link in {f}: {target}"));
            }
        }
    }

    let tracked: BTreeSet<String> = files.iter().cloned().collect();
    let mut inbound: BTreeSet<String> = BTreeSet::new();
    for f in &files {
        let Ok(text) = std::fs::read_to_string(root.join(f)) else {
            continue;
        };
        let dir = Path::new(f).parent().unwrap_or(Path::new(""));
        for target in links(&text) {
            let resolved = normalise(&dir.join(&target))
                .to_string_lossy()
                .replace('\\', "/");
            if &resolved != f {
                inbound.insert(resolved);
            }
        }
        if GENERATED.contains(&f.as_str()) {
            continue;
        }
        let mut fence = false;
        for line in text.lines() {
            if line.trim_start().starts_with("```") {
                fence = !fence;
                continue;
            }
            if fence {
                continue;
            }
            for span in plain_code_spans(line) {
                if resolves_to(&span, f, &tracked).is_some() {
                    bad.push(format!(
                        "unlinked reference in {f}: `{span}` (make it a link)"
                    ));
                }
            }
        }
    }
    for f in &files {
        if !inbound.contains(f) && !NO_INBOUND_NEEDED.contains(&f.as_str()) {
            bad.push(format!(
                "no incoming link to {f} (link it from a related document)"
            ));
        }
    }

    for (rel, max) in [
        ("AGENTS.md", MAX_AGENTS_LINES),
        (".agent/LOG.md", MAX_LOG_LINES),
    ] {
        let n = line_count(root, rel);
        if n > max {
            bad.push(format!("{rel} has {n} lines, the limit is {max}"));
        }
    }
    Ok(bad)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_rules() {
        assert!(covers("docs/archive/", "docs/archive/a/b.md"));
        assert!(covers("tools/visual-qa/**", "tools/visual-qa/x/y.md"));
        assert!(covers("AGENTS.md", "AGENTS.md"));
        assert!(!covers("docs/vision/", "docs/visionary.md"));
        assert!(!covers("AGENTS.md", "docs/AGENTS.md"));
    }

    #[test]
    fn tokens_skip_phrases() {
        let t = index_tokens("| `a/b.md`, `cargo xtask x` | `docs/` |");
        assert_eq!(t, vec!["a/b.md".to_string(), "docs/".to_string()]);
    }

    #[test]
    fn links_skip_urls_and_fragments() {
        let l = links("[a](x/y.md#s) [b](https://e.com/p) [c](#top) [d](../z.md)");
        assert_eq!(l, vec!["x/y.md".to_string(), "../z.md".to_string()]);
    }

    #[test]
    fn links_are_removed_before_span_search() {
        let l = "see [`a/b.md`](a/b.md) and `c/d.md`";
        assert_eq!(plain_code_spans(l), vec!["c/d.md".to_string()]);
    }

    #[test]
    fn resolves_relative_and_root_paths() {
        let tracked: BTreeSet<String> = ["docs/a.md", "docs/x/b.md", "AGENTS.md"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(
            resolves_to("b.md", "docs/x/c.md", &tracked),
            Some("docs/x/b.md".to_string())
        );
        assert_eq!(
            resolves_to("docs/a.md", "AGENTS.md", &tracked),
            Some("docs/a.md".to_string())
        );
        assert_eq!(resolves_to("a.md", "docs/a.md", &tracked), None);
        assert_eq!(resolves_to("x.rs", "docs/a.md", &tracked), None);
    }

    #[test]
    fn normalise_resolves_parent() {
        assert_eq!(
            normalise(Path::new("docs/a/../b/c.md")),
            PathBuf::from("docs/b/c.md")
        );
    }
}
