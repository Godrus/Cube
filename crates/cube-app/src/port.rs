//! The session as an engine-neutral port, and the Godot 4 library entry point.
//!
//! This is the composition root of the first world (DEC-092): it builds a [`Session`] from a
//! directory and a seed, wraps it as a [`SessionPort`] and installs the factory in the thin Godot
//! adapter. It contains no algorithm of its own.

use std::path::Path;
use std::sync::Arc;

use cube_client::{Controls, DeviceClass, FrameOut, Intent, Pose, SessionPort, ViewProfile};
use cube_core::{CellKey, GeneratorVersion, WorldSeed};
use cube_persistence::{STRUCTURE_VERSION, WorldRecord};
use cube_server::{LocalHost, StaticArea};

use crate::Session;

/// Radius of the static bounded area of the first world, in bricks.
pub const FIRST_WORLD_RADIUS: i64 = 5;

/// A session behind the port an adapter drives.
pub struct PortSession(pub Session);

impl SessionPort for PortSession {
    fn frame(&mut self, c: &Controls) -> Result<FrameOut, String> {
        let mut intents = Vec::new();
        if c.look_yaw != 0.0 || c.look_pitch != 0.0 {
            intents.push((
                Intent::Look {
                    yaw: c.look_yaw,
                    pitch: c.look_pitch,
                },
                0.0,
            ));
        }
        if c.forward != 0 || c.right != 0 || c.up != 0 {
            intents.push((
                Intent::Move {
                    forward: c.forward,
                    right: c.right,
                    up: c.up,
                },
                c.dt,
            ));
        }
        let f = self.0.frame(&intents).map_err(|e| e.to_string())?;
        Ok(FrameOut {
            upsert: f.upsert,
            remove: f.remove,
            pending: f.report.pending,
        })
    }

    fn edit(&mut self, place: Option<u16>) -> Result<bool, String> {
        self.0.edit(60.0, place).map_err(|e| e.to_string())
    }

    fn pose(&self) -> Pose {
        let c = self.0.camera();
        Pose {
            pos: c.pos,
            yaw: c.yaw,
            pitch: c.pitch,
        }
    }

    fn shutdown(&mut self) -> Vec<CellKey> {
        self.0.stop().unwrap_or_default()
    }
}

/// Opens (or creates) the first world in `dir` with `seed` and starts a session above its
/// surface, looking down at a gentle angle.
pub fn open_first_world(
    dir: &Path,
    seed: u64,
    create: bool,
) -> Result<Box<dyn SessionPort>, String> {
    let record = WorldRecord {
        world_id: 1,
        seed: WorldSeed::new(seed),
        generator_version: GeneratorVersion::new(1),
        structure_version: STRUCTURE_VERSION,
    };
    let area = StaticArea::around_origin(&record, FIRST_WORLD_RADIUS);
    let host = if create {
        LocalHost::create(dir, record, area)
    } else {
        LocalHost::open(dir, area)
    }
    .map_err(|e| e.to_string())?;
    let host = Arc::new(host);
    let h = host.surface_height(0, 0) as f64;
    let session = Session::new(
        host,
        ViewProfile::of(DeviceClass::Desktop),
        [0.5, h + 14.0, 0.5],
    )
    .map_err(|e| e.to_string())?;
    // Tilt down so the first picture shows the ground (no frame is run here: the first frame
    // belongs to the engine, which needs every batch of geometry).
    session.look(0.0, -0.6);
    Ok(Box::new(PortSession(session)))
}
