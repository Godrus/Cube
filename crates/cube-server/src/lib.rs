//! Cube headless server (experimental spike).
//!
//! Owns the authoritative asset (via `cube-editing`), speaks a tiny line-based text protocol.
//! No authentication, no TLS, no concurrency, no prediction/reconciliation: this only proves
//! that the Rust crates run headless without Godot. Bind to loopback only.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::Path;

use cube_core::{AssetId, DirtyRegion, GeometryAsset, LayerId, Region, VoxelPos};
use cube_editing::{EditError, EditOp, EditOutcome, Editor};
use cube_processing::ProcessingSnapshot;

const MAX_LINE: u64 = 4096;

pub struct Server {
    editor: Editor,
    layer: LayerId,
    shutdown: bool,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        let mut editor = Editor::new(GeometryAsset::new(AssetId(1)));
        let layer = editor.add_voxel_layer(0.1);
        Self {
            editor,
            layer,
            shutdown: false,
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    /// Handles one protocol line and returns one response line (no trailing newline).
    pub fn handle_line(&mut self, line: &str) -> String {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let Some(cmd) = parts.first() else {
            return err("SRV-0001", "empty command");
        };
        match cmd.to_ascii_uppercase().as_str() {
            "PING" => "OK pong".into(),
            "STATUS" => self.status(),
            "FILL" => match parse_ints(&parts[1..], 7) {
                Some(v) if (0..=u16::MAX as i64).contains(&v[6]) => {
                    let r = region(&v[..6]);
                    let op = EditOp::FillRegion {
                        layer: self.layer,
                        region: r,
                        material: v[6] as u16,
                    };
                    edit_result(self.editor.execute(op))
                }
                _ => err(
                    "SRV-0002",
                    "usage: FILL x0 y0 z0 x1 y1 z1 material(0..65535)",
                ),
            },
            "UNDO" => {
                let r = self.editor.undo();
                edit_result(r)
            }
            "REDO" => {
                let r = self.editor.redo();
                edit_result(r)
            }
            "CELLS" => match parse_ints(&parts[1..], 6) {
                Some(v) => {
                    match ProcessingSnapshot::capture(self.editor.asset(), self.layer, region(&v)) {
                        Ok(s) => format!("OK cells={}", s.voxel_cells().len()),
                        Err(e) => err(e.code(), &e.to_string()),
                    }
                }
                None => err("SRV-0002", "usage: CELLS x0 y0 z0 x1 y1 z1"),
            },
            "SAVE" => match parts.get(1) {
                Some(p) => match cube_persistence::save_asset(Path::new(p), self.editor.asset()) {
                    Ok(()) => "OK saved".into(),
                    Err(e) => err(e.code(), &e.to_string()),
                },
                None => err("SRV-0002", "usage: SAVE path"),
            },
            "LOAD" => match parts.get(1) {
                Some(p) => match cube_persistence::load_asset(Path::new(p)) {
                    Ok(asset) => match asset.layers().first().map(|l| l.id()) {
                        Some(layer) => {
                            self.layer = layer;
                            self.editor.replace_asset(asset);
                            self.status()
                        }
                        None => err("SRV-0003", "loaded asset has no layer"),
                    },
                    Err(e) => err(e.code(), &e.to_string()),
                },
                None => err("SRV-0002", "usage: LOAD path"),
            },
            "SHUTDOWN" => {
                self.shutdown = true;
                "OK bye".into()
            }
            _ => err("SRV-0001", "unknown command"),
        }
    }

    fn status(&self) -> String {
        let a = self.editor.asset();
        let lt = a.layer_token(self.layer).map(|t| t.value).unwrap_or(0);
        format!(
            "OK epoch={} asset_rev={} layer_rev={} bricks={} can_undo={} can_redo={}",
            a.epoch(),
            a.asset_token().value,
            lt,
            a.layer(self.layer).map(|l| l.storage().len()).unwrap_or(0),
            self.editor.can_undo(),
            self.editor.can_redo()
        )
    }
}

fn edit_result(r: Result<EditOutcome, EditError>) -> String {
    match r {
        Ok(o) => format!(
            "OK asset_rev={} layer_rev={} changed={} dirty={}",
            o.asset_token.value,
            o.layer_token.value,
            o.changed_voxels,
            dirty_list(&o.dirty)
        ),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

fn region(v: &[i64]) -> Region {
    let c = |i: usize| v[i].clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    Region::new(
        VoxelPos::new(c(0), c(1), c(2)),
        VoxelPos::new(c(3), c(4), c(5)),
    )
}

fn parse_ints(parts: &[&str], n: usize) -> Option<Vec<i64>> {
    if parts.len() != n {
        return None;
    }
    parts.iter().map(|p| p.parse::<i64>().ok()).collect()
}

fn dirty_list(d: &[DirtyRegion]) -> String {
    if d.is_empty() {
        return "none".into();
    }
    d.iter()
        .map(|d| {
            let r = d.region;
            format!(
                "{},{},{},{},{},{}",
                r.min.x, r.min.y, r.min.z, r.max.x, r.max.y, r.max.z
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn err(code: &str, msg: &str) -> String {
    format!("ERR {code} {msg}")
}

/// Serves connections one at a time until `SHUTDOWN` or `max_connections` is reached.
pub fn serve(
    listener: &TcpListener,
    server: &mut Server,
    max_connections: Option<usize>,
) -> std::io::Result<()> {
    for (served, stream) in listener.incoming().enumerate() {
        let mut stream = stream?;
        let mut reader = BufReader::new(stream.try_clone()?);
        loop {
            let mut line = String::new();
            let n = (&mut reader).take(MAX_LINE).read_line(&mut line)?;
            if n == 0 {
                break;
            }
            if !line.ends_with('\n') && n as u64 >= MAX_LINE {
                writeln!(stream, "{}", err("SRV-0004", "line too long"))?;
                break;
            }
            if line.trim().eq_ignore_ascii_case("QUIT") {
                break;
            }
            let reply = server.handle_line(line.trim());
            writeln!(stream, "{reply}")?;
            if server.is_shutdown() {
                return Ok(());
            }
        }
        if max_connections.is_some_and(|m| served + 1 >= m) {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_basic_flow() {
        let mut s = Server::new();
        assert_eq!(s.handle_line("PING"), "OK pong");
        let r = s.handle_line("FILL 0 0 0 2 2 2 5");
        assert!(
            r.starts_with("OK asset_rev=2 layer_rev=1 changed=8 dirty=0,0,0,2,2,2"),
            "{r}"
        );
        assert_eq!(s.handle_line("CELLS 0 0 0 16 16 16"), "OK cells=8");
        assert!(s.handle_line("UNDO").starts_with("OK"));
        assert_eq!(s.handle_line("CELLS 0 0 0 16 16 16"), "OK cells=0");
        assert!(s.handle_line("REDO").starts_with("OK"));
        assert_eq!(s.handle_line("CELLS 0 0 0 16 16 16"), "OK cells=8");
    }

    #[test]
    fn protocol_errors_are_typed_and_state_unchanged() {
        let mut s = Server::new();
        let before = s.handle_line("STATUS");
        assert!(s
            .handle_line("FILL 0 0 0 2 2 2 70000")
            .starts_with("ERR SRV-0002"));
        assert!(s
            .handle_line("FILL 1 1 1 1 5 5 1")
            .starts_with("ERR EDIT-0001"));
        assert!(s.handle_line("UNDO").starts_with("ERR EDIT-0002"));
        assert!(s.handle_line("NOPE").starts_with("ERR SRV-0001"));
        assert!(s.handle_line("").starts_with("ERR SRV-0001"));
        assert_eq!(s.handle_line("STATUS"), before);
    }
}
