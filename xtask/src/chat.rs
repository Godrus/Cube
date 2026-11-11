//! Shared agent channel. `cargo xtask say` appends one line to `.agent/live/CHAT.md`;
//! `cargo xtask chat` prints the last lines. The file is not tracked by git and always lives
//! in the main checkout, so agents in different worktrees see the same file.
//!
//! One line is written with a single append, so two agents writing at the same moment do not
//! overwrite each other (editing the file by hand or with an editor tool can lose messages).

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// How many trailing lines `chat` prints by default.
const DEFAULT_TAIL: usize = 20;

/// Main checkout root: the parent of git's common directory (same for every worktree).
fn main_checkout(fallback: &Path) -> PathBuf {
    let out = Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .current_dir(fallback)
        .output();
    let dir = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => String::new(),
    };
    match Path::new(&dir).parent() {
        Some(parent) if !dir.is_empty() => parent.to_path_buf(),
        _ => fallback.to_path_buf(),
    }
}

fn chat_file(root: &Path) -> PathBuf {
    main_checkout(root)
        .join(".agent")
        .join("live")
        .join("CHAT.md")
}

/// Converts days since 1970-01-01 to (year, month, day), proleptic Gregorian.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `YYYY-MM-DDTHH:MMZ` (UTC) for a Unix time in seconds.
fn stamp(unix_secs: u64) -> String {
    let days = (unix_secs / 86_400) as i64;
    let rem = unix_secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}Z",
        rem / 3_600,
        rem % 3_600 / 60
    )
}

/// One chat line: `<stamp> <agent>: <text>` with the text on a single line.
fn format_line(unix_secs: u64, agent: &str, text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    format!("{} {agent}: {flat}\n", stamp(unix_secs))
}

/// `say [--as <id>] <text>`: the id comes from `--as` or the `CUBE_AGENT` variable.
pub fn say(root: &Path, args: &[String]) -> Result<String, String> {
    let mut agent = std::env::var("CUBE_AGENT")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let mut words: Vec<&str> = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--as" {
            agent = it.next().cloned();
        } else {
            words.push(a);
        }
    }
    let agent = agent.ok_or("agent id missing: pass --as <id> or set CUBE_AGENT")?;
    if agent.contains(char::is_whitespace) {
        return Err("agent id must not contain spaces".into());
    }
    let text = words.join(" ");
    if text.trim().is_empty() {
        return Err("message is empty".into());
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let line = format_line(now, &agent, &text);
    let path = chat_file(root);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    f.write_all(line.as_bytes())
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(line.trim_end().to_string())
}

/// `chat [N]`: the last N lines (default 20), or a note that the channel is empty.
pub fn chat(root: &Path, args: &[String]) -> Result<String, String> {
    let n = match args.first() {
        Some(s) => s
            .parse::<usize>()
            .map_err(|_| format!("not a number: {s}"))?,
        None => DEFAULT_TAIL,
    };
    let path = chat_file(root);
    let Ok(body) = fs::read_to_string(&path) else {
        return Ok(format!("(empty: {} does not exist yet)", path.display()));
    };
    let lines: Vec<&str> = body.lines().collect();
    let from = lines.len().saturating_sub(n);
    Ok(lines[from..].join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamp_matches_known_dates() {
        assert_eq!(stamp(0), "1970-01-01T00:00Z");
        assert_eq!(stamp(951_782_400), "2000-02-29T00:00Z");
        assert_eq!(stamp(1_794_410_700), "2026-11-11T15:25Z");
    }

    #[test]
    fn line_is_single_and_trimmed() {
        let l = format_line(0, "claude-1", "a\n  b\tc ");
        assert_eq!(l, "1970-01-01T00:00Z claude-1: a b c\n");
    }
}
