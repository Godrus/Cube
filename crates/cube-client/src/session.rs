//! The port an engine adapter drives (engine-neutral): per-frame controls in, buffers out.
//!
//! The adapter translates engine input into [`Controls`] and [`FrameOut`] buffers into scene
//! nodes; it owns neither the world nor any algorithm. A concrete session (the composition root
//! of a local or remote world) implements [`SessionPort`] and is handed to the adapter at start
//! up. Nothing here names an engine.

use cube_core::CellKey;

use crate::MeshBuffers;

/// The integer coordinates of a cell key, for engines that key their nodes by brick.
pub trait CellKeyExt {
    /// `[x, y, z]` of the brick.
    fn coords(&self) -> [i64; 3];
}

impl CellKeyExt for CellKey {
    fn coords(&self) -> [i64; 3] {
        let b = self.brick();
        [b.x, b.y, b.z]
    }
}

/// One frame of player input, already reduced to what the client understands.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Controls {
    /// Forward (+1) or backward (-1).
    pub forward: i8,
    /// Right (+1) or left (-1).
    pub right: i8,
    /// Up (+1) or down (-1).
    pub up: i8,
    /// Turn left (positive) or right, radians.
    pub look_yaw: f32,
    /// Look up (positive) or down, radians.
    pub look_pitch: f32,
    /// Seconds since the last frame.
    pub dt: f32,
}

/// What the engine must do after a frame.
#[derive(Default)]
pub struct FrameOut {
    /// Bricks to create or replace.
    pub upsert: Vec<(CellKey, MeshBuffers)>,
    /// Bricks to remove.
    pub remove: Vec<CellKey>,
    /// Bricks of the active region still waiting for their first geometry.
    pub pending: usize,
}

/// Camera pose for the engine camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    /// Position in voxel units (Y up).
    pub pos: [f64; 3],
    /// Rotation around the vertical axis (radians).
    pub yaw: f32,
    /// Rotation up and down (radians).
    pub pitch: f32,
}

/// Counters an engine may show (streaming and memory), all read-only.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SessionStats {
    /// Bricks resident in the authoritative world.
    pub resident: u64,
    /// Of those, bricks with user edits (never unloaded).
    pub edited: u64,
    /// Bricks generated or reloaded since the start.
    pub loads: u64,
    /// Bricks unloaded since the start.
    pub evictions: u64,
    /// Cached derived entries (faces) in Processing.
    pub derived_cached: u64,
    /// Bricks with geometry in the client's active region.
    pub active: u64,
}

/// The surface an adapter sees of a world.
pub trait SessionPort: Send {
    /// Runs one frame.
    fn frame(&mut self, controls: &Controls) -> Result<FrameOut, String>;
    /// Removes (`None`) or places (`Some(material)`) the block under the crosshair; `Ok(false)`
    /// if nothing was in reach. The change is a request answered by the authoritative side.
    fn edit(&mut self, place: Option<u16>) -> Result<bool, String>;
    /// The current camera pose.
    fn pose(&self) -> Pose;
    /// Stops the session; returns bricks the engine must drop.
    fn shutdown(&mut self) -> Vec<CellKey>;
    /// Switches between walking (gravity, collision) and free flight. Sessions without a walking
    /// player ignore it.
    fn set_walk(&mut self, _on: bool) {}
    /// Chooses the third-person camera (walking only).
    fn set_third_person(&mut self, _on: bool) {}
    /// `true` while the player walks.
    fn walking(&self) -> bool {
        false
    }
    /// Feet position of the walking player, if any.
    fn player_feet(&self) -> Option<[f64; 3]> {
        None
    }
    /// The walking player stands on something.
    fn grounded(&self) -> bool {
        false
    }
    /// Streaming and memory counters.
    fn stats(&self) -> SessionStats {
        SessionStats::default()
    }
}
