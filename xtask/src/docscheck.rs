//! Documentation check. `.agent/INDEX.md` is the allow-list of Markdown files; this check
//! fails when a tracked `.md` file is not covered by it, when a path listed in it does not
//! exist, when a relative Markdown link is broken, or when the short entry files grow.
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
    fn normalise_resolves_parent() {
        assert_eq!(
            normalise(Path::new("docs/a/../b/c.md")),
            PathBuf::from("docs/b/c.md")
        );
    }
}
