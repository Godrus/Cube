//! Shared agent channel and roster. `cargo xtask join` registers an agent (tool, model, task) and
//! gives it a numbered id; `who` lists the active agents; `say` appends one line to
//! `.agent/live/CHAT.md` (only for a registered id, the line names tool, model and task);
//! `chat` prints the last lines. All files are not tracked by git and always live in the main
//! checkout, so agents in different worktrees or sessions see the same files.
//!
//! One line is written with a single append, so two agents writing at the same moment do not
//! overwrite each other (editing the files by hand or with an editor tool can lose lines).

use std::collections::BTreeMap;
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

fn live_file(root: &Path, name: &str) -> PathBuf {
    main_checkout(root).join(".agent").join("live").join(name)
}

fn chat_file(root: &Path) -> PathBuf {
    live_file(root, "CHAT.md")
}

fn roster_file(root: &Path) -> PathBuf {
    live_file(root, "ROSTER.md")
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

/// A registered agent (latest state from the roster).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Agent {
    tool: String,
    model: String,
    task: String,
    active: bool,
}

/// Human name of a tool id, used in the work log and in `who`.
fn tool_name(tool: &str) -> String {
    match tool {
        "claude" => "Claude Code".into(),
        "cursor" => "Cursor".into(),
        "codex" => "Codex".into(),
        "chatgpt" => "ChatGPT".into(),
        other => other.into(),
    }
}

/// `Claude Code, sonnet-5.5`: the descriptor written after the id in the work log.
fn descriptor(a: &Agent) -> String {
    if a.model == "?" {
        tool_name(&a.tool)
    } else {
        format!("{}, {}", tool_name(&a.tool), a.model)
    }
}

/// Reads the roster (`time|id|event|tool|model|task`, newest event last) into the latest state.
fn parse_roster(text: &str) -> BTreeMap<String, Agent> {
    let mut map: BTreeMap<String, Agent> = BTreeMap::new();
    for line in text.lines() {
        let f: Vec<&str> = line.split('|').collect();
        if f.len() != 6 {
            continue;
        }
        let id = f[1].to_string();
        match f[2] {
            "join" => {
                map.insert(
                    id,
                    Agent {
                        tool: f[3].into(),
                        model: f[4].into(),
                        task: f[5].into(),
                        active: true,
                    },
                );
            }
            "task" => {
                if let Some(a) = map.get_mut(&id) {
                    a.task = f[5].into();
                }
            }
            "left" => {
                if let Some(a) = map.get_mut(&id) {
                    a.active = false;
                }
            }
            _ => {}
        }
    }
    map
}

fn load_roster(root: &Path) -> BTreeMap<String, Agent> {
    fs::read_to_string(roster_file(root)).map_or_else(|_| BTreeMap::new(), |t| parse_roster(&t))
}

/// Next number for `tool` that no known text uses yet (`claude-3` after `claude-2`).
fn next_number(tool: &str, texts: &[String]) -> u32 {
    let prefix = format!("{tool}-");
    let mut max = 0;
    for t in texts {
        for word in t.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
            if let Some(n) = word
                .strip_prefix(&prefix)
                .and_then(|r| r.parse::<u32>().ok())
            {
                max = max.max(n);
            }
        }
    }
    max + 1
}

fn now_secs() -> Result<u64, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs())
}

/// Appends one line to a file under `.agent/live`, creating the folder.
fn append(path: &Path, line: &str) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    f.write_all(line.as_bytes())
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// Value of `--name <value>` in `args` (the pair is removed).
fn take_flag(args: &mut Vec<String>, name: &str) -> Option<String> {
    let i = args.iter().position(|a| a == name)?;
    args.remove(i);
    (i < args.len()).then(|| args.remove(i))
}

fn clean_field(name: &str, v: &str) -> Result<String, String> {
    let v = v.trim();
    if v.is_empty() || v.contains('|') || v.contains(char::is_whitespace) {
        return Err(format!(
            "{name} must be one word without | or spaces: '{v}'"
        ));
    }
    Ok(v.to_string())
}

/// The id of the calling agent: `--as <id>` or the `CUBE_AGENT` variable.
fn caller(args: &mut Vec<String>) -> Result<String, String> {
    take_flag(args, "--as")
        .or_else(|| std::env::var("CUBE_AGENT").ok())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "agent id missing: pass --as <id> or set CUBE_AGENT".to_string())
}

fn roster_line(unix: u64, id: &str, event: &str, a: &Agent) -> String {
    format!(
        "{}|{id}|{event}|{}|{}|{}\n",
        stamp(unix),
        a.tool,
        a.model,
        a.task
    )
}

fn not_registered(id: &str) -> String {
    format!(
        "agent id '{id}' is not registered: run `cargo xtask join --tool <claude|cursor|codex|chatgpt> --model <model> --task <ID>`"
    )
}

/// `join --tool <t> [--model <m>] [--task <ID>] [--id <id>]`: registers the agent.
pub fn join(root: &Path, mut args: Vec<String>) -> Result<String, String> {
    let tool = take_flag(&mut args, "--tool")
        .ok_or("--tool is required (claude, cursor, codex, chatgpt)")?;
    let tool = clean_field("--tool", &tool)?.to_lowercase();
    let model = take_flag(&mut args, "--model").unwrap_or_else(|| "?".into());
    let task = take_flag(&mut args, "--task").unwrap_or_else(|| "-".into());
    let agent = Agent {
        tool: tool.clone(),
        model: clean_field("--model", &model)?,
        task: clean_field("--task", &task)?,
        active: true,
    };
    let roster = load_roster(root);
    let id = match take_flag(&mut args, "--id") {
        Some(id) => {
            let id = clean_field("--id", &id)?;
            if roster.get(&id).is_some_and(|a| a.active) {
                return Err(format!(
                    "{id} is already active; leave it first or pick another id"
                ));
            }
            id
        }
        None => {
            let main = main_checkout(root);
            let mut texts = vec![fs::read_to_string(roster_file(root)).unwrap_or_default()];
            for rel in [".agent/LOG.md", ".agent/live/CHAT.md"] {
                texts.push(fs::read_to_string(main.join(rel)).unwrap_or_default());
            }
            format!("{tool}-{}", next_number(&tool, &texts))
        }
    };
    append(
        &roster_file(root),
        &roster_line(now_secs()?, &id, "join", &agent),
    )?;
    let d = descriptor(&agent);
    Ok(format!(
        "{id} ({d}), task {}. Use --as {id} or set CUBE_AGENT={id}. Work log column: {id} ({d})",
        agent.task
    ))
}

/// `task --as <id> <ID>`: changes the task shown in the roster and in chat lines.
pub fn task(root: &Path, mut args: Vec<String>) -> Result<String, String> {
    let id = caller(&mut args)?;
    let task = clean_field("task", args.first().ok_or("task id missing")?)?;
    let roster = load_roster(root);
    let mut a = roster
        .get(&id)
        .filter(|a| a.active)
        .cloned()
        .ok_or_else(|| not_registered(&id))?;
    a.task = task.clone();
    append(
        &roster_file(root),
        &roster_line(now_secs()?, &id, "task", &a),
    )?;
    Ok(format!("{id}: task {task}"))
}

/// `leave --as <id>`: marks the agent as gone.
pub fn leave(root: &Path, mut args: Vec<String>) -> Result<String, String> {
    let id = caller(&mut args)?;
    let roster = load_roster(root);
    let a = roster.get(&id).ok_or_else(|| not_registered(&id))?;
    append(
        &roster_file(root),
        &roster_line(now_secs()?, &id, "left", a),
    )?;
    Ok(format!("{id} left"))
}

/// `who [<id>]`: active agents, or the work-log descriptor of one id.
pub fn who(root: &Path, args: &[String]) -> Result<String, String> {
    let roster = load_roster(root);
    if let Some(id) = args.first() {
        let a = roster.get(id).ok_or_else(|| not_registered(id))?;
        return Ok(format!("{id} ({})", descriptor(a)));
    }
    let rows: Vec<String> = roster
        .iter()
        .filter(|(_, a)| a.active)
        .map(|(id, a)| format!("{id} | {} | {}", descriptor(a), a.task))
        .collect();
    if rows.is_empty() {
        Ok("(no active agents: `cargo xtask join --tool <t> --model <m> --task <ID>`)".into())
    } else {
        Ok(format!("id | tool, model | task\n{}", rows.join("\n")))
    }
}

/// One chat line: `<stamp> <id> [<tool>, <model>, <task>]: <text>` with the text on one line.
fn format_line(unix_secs: u64, id: &str, a: &Agent, text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    format!(
        "{} {id} [{}, {}]: {flat}\n",
        stamp(unix_secs),
        descriptor(a),
        a.task
    )
}

/// `say [--as <id>] <text>`: the id must be registered with `join`.
pub fn say(root: &Path, mut args: Vec<String>) -> Result<String, String> {
    let id = caller(&mut args)?;
    let roster = load_roster(root);
    let agent = roster
        .get(&id)
        .filter(|a| a.active)
        .ok_or_else(|| not_registered(&id))?;
    let text = args.join(" ");
    if text.trim().is_empty() {
        return Err("message is empty".into());
    }
    let line = format_line(now_secs()?, &id, agent, &text);
    append(&chat_file(root), &line)?;
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

    fn sample() -> Agent {
        Agent {
            tool: "claude".into(),
            model: "sonnet-5.5".into(),
            task: "MAT-01".into(),
            active: true,
        }
    }

    #[test]
    fn stamp_matches_known_dates() {
        assert_eq!(stamp(0), "1970-01-01T00:00Z");
        assert_eq!(stamp(951_782_400), "2000-02-29T00:00Z");
        assert_eq!(stamp(1_794_410_700), "2026-11-11T15:25Z");
    }

    #[test]
    fn line_names_agent_tool_model_and_task() {
        let l = format_line(0, "claude-1", &sample(), "a\n  b\tc ");
        assert_eq!(
            l,
            "1970-01-01T00:00Z claude-1 [Claude Code, sonnet-5.5, MAT-01]: a b c\n"
        );
    }

    #[test]
    fn roster_keeps_latest_state() {
        let text = "t|claude-1|join|claude|opus|A\nt|claude-1|task|claude|opus|B\nt|cursor-1|join|cursor|?|C\nt|cursor-1|left|cursor|?|C\n";
        let r = parse_roster(text);
        assert_eq!(r["claude-1"].task, "B");
        assert!(r["claude-1"].active);
        assert!(!r["cursor-1"].active);
        assert_eq!(descriptor(&r["cursor-1"]), "Cursor");
    }

    #[test]
    fn numbers_skip_ids_seen_anywhere() {
        let texts = vec![
            "x claude-2 y".to_string(),
            "| claude-7 (Claude Code) |".to_string(),
        ];
        assert_eq!(next_number("claude", &texts), 8);
        assert_eq!(next_number("cursor", &texts), 1);
    }

    #[test]
    fn flags_are_taken_out_of_args() {
        let mut a: Vec<String> = ["--tool", "claude", "x"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(take_flag(&mut a, "--tool"), Some("claude".to_string()));
        assert_eq!(a, vec!["x".to_string()]);
        assert!(clean_field("f", "a b").is_err());
    }
}
