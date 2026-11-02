//! Headless Cube server entry point. Never starts Godot.

use std::net::TcpListener;
use std::process::ExitCode;

use cube_server::{serve, Server};

const DEFAULT_ADDR: &str = "127.0.0.1:7878";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--selftest") => selftest(),
        Some("--listen") => run(args.get(1).map(String::as_str).unwrap_or(DEFAULT_ADDR)),
        None => run(DEFAULT_ADDR),
        Some(_) => {
            eprintln!("usage: cube-server [--listen ADDR | --selftest]\n  default ADDR is {DEFAULT_ADDR} (loopback only; no authentication)");
            ExitCode::from(2)
        }
    }
}

fn run(addr: &str) -> ExitCode {
    let listener = match TcpListener::bind(addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("cannot bind {addr}: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("cube-server listening on {addr} (send SHUTDOWN to stop)");
    let mut server = Server::new();
    match serve(&listener, &mut server, None) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("server error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn selftest() -> ExitCode {
    let mut s = Server::new();
    let script = [
        ("FILL -8 0 -8 8 4 8 3", "OK"),
        ("CELLS -16 0 -16 16 16 16", "OK cells=1024"),
        ("UNDO", "OK"),
        ("REDO", "OK"),
        ("STATUS", "OK"),
    ];
    for (cmd, want) in script {
        let got = s.handle_line(cmd);
        println!("> {cmd}\n< {got}");
        if !got.starts_with(want) {
            eprintln!("selftest failed at '{cmd}'");
            return ExitCode::FAILURE;
        }
    }
    println!("selftest ok");
    ExitCode::SUCCESS
}
