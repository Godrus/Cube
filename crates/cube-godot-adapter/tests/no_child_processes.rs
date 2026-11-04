//! Guard: the Godot client must not start child processes or read system properties through a shell, and
//! the client export presets must not ship the test scripts. `OS.execute("getprop")` crashed the ARM64
//! translated emulator (report, section 26); this test stops it from coming back.

use std::fs;
use std::path::{Path, PathBuf};

fn client_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot-client")
}

fn gd_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if p.is_dir() {
            // Build outputs and the editor cache are not sources.
            if !matches!(name.as_str(), "bin" | "build" | ".godot" | ".import") {
                gd_files(&p, out);
            }
        } else if name.ends_with(".gd") {
            out.push(p);
        }
    }
}

/// Source text of a GDScript file without `#` comments (a `#` inside a string literal is kept).
fn code_only(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        let mut quote: Option<char> = None;
        let mut kept = String::new();
        for c in line.chars() {
            match quote {
                Some(q) if c == q => quote = None,
                Some(_) => {}
                None if c == '"' || c == '\'' => quote = Some(c),
                None if c == '#' => break,
                None => {}
            }
            kept.push(c);
        }
        out.push_str(&kept);
        out.push('\n');
    }
    out
}

const FORBIDDEN: [&str; 7] = [
    "OS.execute",
    "OS.execute_with_pipe",
    "OS.create_process",
    "OS.create_instance",
    "OS.shell_open",
    "OS.shell_show_in_file_manager",
    "getprop",
];

#[test]
fn no_gdscript_starts_a_child_process_or_reads_system_properties() {
    let mut files = Vec::new();
    gd_files(&client_dir(), &mut files);
    assert!(files.len() >= 3, "found the client scripts: {files:?}");
    for f in files {
        let code = code_only(&fs::read_to_string(&f).unwrap());
        for bad in FORBIDDEN {
            assert!(
                !code.contains(bad),
                "{} must not use `{bad}` (child processes crash the ARM64 translated emulator, and a runtime must not shell out)",
                f.display()
            );
        }
    }
}

#[test]
fn the_runtime_client_script_contains_no_test_code() {
    let code = code_only(&fs::read_to_string(client_dir().join("world.gd")).unwrap());
    for hook in [
        "WORLDTEST",
        "worldtest",
        "--world-smoke",
        "_smoke",
        "soak",
        "world_test",
        "noexec",
    ] {
        assert!(
            !code.contains(hook),
            "world.gd must stay free of test hooks, found `{hook}`"
        );
    }
    let scene = fs::read_to_string(client_dir().join("world.tscn")).unwrap();
    assert!(scene.contains("res://world.gd") && !scene.contains("world_smoke"));
    let test_scene = fs::read_to_string(client_dir().join("world_test.tscn")).unwrap();
    assert!(test_scene.contains("res://world_smoke.gd"));
}

#[test]
fn client_export_presets_do_not_ship_test_scripts() {
    let presets = fs::read_to_string(client_dir().join("export_presets.cfg")).unwrap();
    let mut clients = 0;
    for block in presets.split("\n[preset.") {
        let field = |key: &str| -> Option<String> {
            block
                .lines()
                .find(|l| l.starts_with(&format!("{key}=")))
                .map(|l| l[key.len() + 1..].trim_matches('"').to_string())
        };
        let Some(features) = field("custom_features") else {
            continue;
        };
        if features.split(',').any(|f| f == "worldclient") {
            clients += 1;
            assert!(
                !features.split(',').any(|f| f == "worldtest" || f == "soak"),
                "client preset has test features: {features}"
            );
            let exclude = field("exclude_filter").unwrap_or_default();
            for test_file in ["world_smoke.gd", "world_test.tscn"] {
                assert!(
                    exclude.contains(test_file),
                    "client preset must exclude {test_file}: {exclude}"
                );
            }
        }
    }
    assert!(clients >= 2, "client presets exist for faces and greedy");
}
