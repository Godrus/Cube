//! Public-API snapshot of cube-editing-api (API-05, contract-change tooling).
//!
//! The snapshot lists every `pub` declaration line of `src/` (test modules excluded), per file.
//! Any difference fails this test. To accept an intended change, regenerate the file with
//! `UPDATE_API_SNAPSHOT=1 cargo test -p cube-editing-api --test api_snapshot` and state the change request
//! in the commit message (integration-and-qa-plan section 5).

use std::fs;
use std::path::{Path, PathBuf};

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

fn current_snapshot() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let mut files = Vec::new();
    rs_files(&src, &mut files);
    files.sort();
    let mut out = String::new();
    for f in files {
        let rel = f
            .strip_prefix(&src)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(&f).unwrap();
        for line in text.lines() {
            let t = line.trim();
            if t.starts_with("mod tests") {
                break;
            }
            if t.starts_with("pub ") {
                let norm = t.split_whitespace().collect::<Vec<_>>().join(" ");
                out.push_str(&format!("{rel}: {norm}\n"));
            }
        }
    }
    out
}

#[test]
fn public_api_matches_the_committed_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("api-snapshot.txt");
    let now = current_snapshot();
    if std::env::var_os("UPDATE_API_SNAPSHOT").is_some() {
        fs::write(&path, &now).unwrap();
        return;
    }
    let committed = fs::read_to_string(&path)
        .expect("api-snapshot.txt is missing; generate it with UPDATE_API_SNAPSHOT=1")
        .replace("\r\n", "\n");
    assert_eq!(
        now, committed,
        "public API changed: review it and regenerate api-snapshot.txt with UPDATE_API_SNAPSHOT=1"
    );
}
