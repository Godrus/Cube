//! Repository tooling. `cargo xtask archcheck` is the architecture check (XT-01).

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("archcheck") => {
            // PS-0 skeleton: the rule set is added by XT-01. The workspace has no
            // production crates yet, so there is nothing to violate.
            println!("archcheck: skeleton (no rules yet)");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("usage: cargo xtask archcheck");
            ExitCode::from(2)
        }
    }
}
