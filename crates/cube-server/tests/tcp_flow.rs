//! Integration: server over a real loopback socket, no Godot involved.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use cube_server::{serve, Server};

fn ask(w: &mut TcpStream, r: &mut BufReader<TcpStream>, cmd: &str) -> String {
    writeln!(w, "{cmd}").unwrap();
    let mut line = String::new();
    r.read_line(&mut line).unwrap();
    line.trim().to_string()
}

#[test]
fn edit_save_load_over_tcp() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let mut server = Server::new();
        serve(&listener, &mut server, Some(1)).unwrap();
    });

    let mut w = TcpStream::connect(addr).unwrap();
    let mut r = BufReader::new(w.try_clone().unwrap());

    assert_eq!(ask(&mut w, &mut r, "PING"), "OK pong");
    let fill = ask(&mut w, &mut r, "FILL -3 -3 -3 3 3 3 9");
    assert!(
        fill.starts_with("OK asset_rev=2 layer_rev=1 changed=216 dirty=-3,-3,-3,3,3,3"),
        "{fill}"
    );

    let mut path = std::env::temp_dir();
    path.push(format!("cube_srv_{}.cube", std::process::id()));
    let p = path.to_string_lossy().replace('\\', "/");
    assert_eq!(ask(&mut w, &mut r, &format!("SAVE {p}")), "OK saved");
    assert!(ask(&mut w, &mut r, "UNDO").starts_with("OK"));
    assert_eq!(
        ask(&mut w, &mut r, "CELLS -16 -16 -16 16 16 16"),
        "OK cells=0"
    );
    let loaded = ask(&mut w, &mut r, &format!("LOAD {p}"));
    assert!(loaded.starts_with("OK epoch=1 "), "{loaded}");
    assert_eq!(
        ask(&mut w, &mut r, "CELLS -16 -16 -16 16 16 16"),
        "OK cells=216"
    );
    assert!(ask(&mut w, &mut r, "LOAD /nonexistent/file.cube").starts_with("ERR PERS-0001"));

    writeln!(w, "QUIT").unwrap();
    handle.join().unwrap();
    let _ = std::fs::remove_file(path);
}
