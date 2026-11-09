//! Repository tooling. `cargo xtask archcheck` is the architecture check (XT-01);
//! `cargo xtask features` checks the feature registry and regenerates its overview.

mod archcheck;
mod features;

use std::path::PathBuf;
use std::process::ExitCode;

fn workspace_root() -> PathBuf {
    // xtask lives in <root>/xtask
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .unwrap_or_default()
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("archcheck") => match archcheck::run(&workspace_root()) {
            Ok(v) if v.is_empty() => {
                println!("archcheck: OK (graph equals ADG Part B edges, no forbidden use)");
                ExitCode::SUCCESS
            }
            Ok(v) => {
                for x in &v {
                    eprintln!("{x}");
                }
                eprintln!("archcheck: {} violation(s)", v.len());
                ExitCode::from(1)
            }
            Err(e) => {
                eprintln!("archcheck: cannot run: {e}");
                ExitCode::from(2)
            }
        },
        Some("features") => {
            let write = match args.next().as_deref() {
                Some("--write") => true,
                Some("--check") | None => false,
                Some(other) => {
                    eprintln!("features: unknown option {other}");
                    return ExitCode::from(2);
                }
            };
            match features::run(&workspace_root(), write) {
                Ok(v) if v.is_empty() => {
                    println!("features: OK");
                    ExitCode::SUCCESS
                }
                Ok(v) => {
                    for x in &v {
                        eprintln!("features: {x}");
                    }
                    ExitCode::from(1)
                }
                Err(e) => {
                    eprintln!("features: cannot run: {e}");
                    ExitCode::from(2)
                }
            }
        }
        _ => {
            eprintln!("usage: cargo xtask archcheck | features [--check|--write]");
            ExitCode::from(2)
        }
    }
}
