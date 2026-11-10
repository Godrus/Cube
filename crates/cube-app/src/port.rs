//! The session as an engine-neutral port, and the Godot 4 library entry point.
//!
//! This is the composition root of the first world (DEC-092): it builds a [`Session`] from a
//! directory and a seed, wraps it as a [`SessionPort`] and installs the factory in the thin Godot
//! adapter. It contains no algorithm of its own.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use cube_client::{Controls, DeviceClass, FrameOut, Intent, Pose, SessionPort, ViewProfile};
use cube_core::{CellKey, GeneratorVersion, WorldSeed};
use cube_persistence::{STRUCTURE_VERSION, WorldRecord};
use cube_server::{LocalHost, StaticArea};

use crate::{PlayerState, Session};

/// Radius of the static bounded area of the first world, in bricks.
pub const FIRST_WORLD_RADIUS: i64 = 5;

/// File next to the world that remembers where the player stood (client data, not world state).
pub const PLAYER_FILE: &str = "player.state";
/// Seconds between automatic saves of the player.
const AUTOSAVE_SECONDS: f32 = 5.0;

/// Text form of a [`PlayerState`]: one `key value...` line each, version first.
pub fn encode_player(s: &PlayerState) -> String {
    format!(
        "cube-player 1\nfeet {} {} {}\nview {} {}\nwalking {}\nthird_person {}\n",
        s.feet[0],
        s.feet[1],
        s.feet[2],
        s.yaw,
        s.pitch,
        u8::from(s.walking),
        u8::from(s.third_person)
    )
}

/// Reads [`encode_player`] output; `None` for anything that is not a complete, finite state.
pub fn decode_player(text: &str) -> Option<PlayerState> {
    let mut lines = text.lines();
    if lines.next()? != "cube-player 1" {
        return None;
    }
    let (mut feet, mut view, mut walking, mut third) = (None, None, None, None);
    for l in lines {
        let mut it = l.split_whitespace();
        match it.next()? {
            "feet" => {
                let v: Vec<f64> = it.map(str::parse).collect::<Result<_, _>>().ok()?;
                feet = <[f64; 3]>::try_from(v).ok();
            }
            "view" => {
                let v: Vec<f32> = it.map(str::parse).collect::<Result<_, _>>().ok()?;
                view = <[f32; 2]>::try_from(v).ok();
            }
            "walking" => walking = Some(it.next()? == "1"),
            "third_person" => third = Some(it.next()? == "1"),
            _ => return None,
        }
    }
    let (feet, view) = (feet?, view?);
    if !feet.iter().all(|c| c.is_finite()) || !view.iter().all(|c| c.is_finite()) {
        return None;
    }
    Some(PlayerState {
        feet,
        yaw: view[0],
        pitch: view[1],
        walking: walking?,
        third_person: third?,
    })
}

/// Writes the player next to the world (temporary file, then rename: a crash never leaves a
/// half-written file).
pub fn save_player(dir: &Path, s: &PlayerState) -> std::io::Result<()> {
    let tmp = dir.join(format!("{PLAYER_FILE}.tmp"));
    std::fs::write(&tmp, encode_player(s))?;
    std::fs::rename(&tmp, dir.join(PLAYER_FILE))
}

/// The saved player of the world in `dir`, if there is a valid one.
pub fn load_player(dir: &Path) -> Option<PlayerState> {
    decode_player(&std::fs::read_to_string(dir.join(PLAYER_FILE)).ok()?)
}

/// A session behind the port an adapter drives.
pub struct PortSession {
    session: Session,
    dir: PathBuf,
    since_save: f32,
}

impl PortSession {
    /// Wraps a session whose world lives in `dir` (the player is saved there).
    pub fn new(session: Session, dir: PathBuf) -> Self {
        Self {
            session,
            dir,
            since_save: 0.0,
        }
    }

    fn save(&self) {
        // A failed save must not stop the game; the world itself is saved by the host.
        let _ = save_player(&self.dir, &self.session.player_state());
    }
}

impl SessionPort for PortSession {
    fn frame(&mut self, c: &Controls) -> Result<FrameOut, String> {
        self.since_save += c.dt;
        if self.since_save >= AUTOSAVE_SECONDS {
            self.since_save = 0.0;
            self.save();
        }
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
        // A walking player needs time to pass even when no key is down (gravity).
        if c.forward != 0 || c.right != 0 || c.up != 0 || self.session.walking() {
            intents.push((
                Intent::Move {
                    forward: c.forward,
                    right: c.right,
                    up: c.up,
                },
                c.dt,
            ));
        }
        let f = self.session.frame(&intents).map_err(|e| e.to_string())?;
        Ok(FrameOut {
            upsert: f.upsert,
            remove: f.remove,
            pending: f.report.pending,
        })
    }

    fn edit(&mut self, place: Option<u16>) -> Result<bool, String> {
        self.session.edit(60.0, place).map_err(|e| e.to_string())
    }

    fn pose(&self) -> Pose {
        let c = self.session.camera();
        Pose {
            pos: c.pos,
            yaw: c.yaw,
            pitch: c.pitch,
        }
    }

    fn shutdown(&mut self) -> Vec<CellKey> {
        self.save();
        self.session.stop().unwrap_or_default()
    }

    fn set_walk(&mut self, on: bool) {
        self.session.set_walk(on);
    }

    fn set_third_person(&mut self, on: bool) {
        self.session.set_third_person(on);
    }

    fn walking(&self) -> bool {
        self.session.walking()
    }

    fn player_feet(&self) -> Option<[f64; 3]> {
        self.session.player_feet()
    }

    fn grounded(&self) -> bool {
        self.session.grounded()
    }

    fn stats(&self) -> cube_client::SessionStats {
        self.session.stats()
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
    if !create {
        // Continue where the player left off (a missing or damaged file means a fresh start).
        if let Some(saved) = load_player(dir) {
            session.restore_player(&saved);
        }
    }
    Ok(Box::new(PortSession::new(session, dir.to_path_buf())))
}
