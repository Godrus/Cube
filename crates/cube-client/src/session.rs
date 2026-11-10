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

/// What the crosshair points at and what the player may do about it.
#[derive(Debug, Clone, PartialEq)]
pub struct TargetInfo {
    /// The solid voxel under the crosshair.
    pub voxel: [i64; 3],
    /// The empty voxel in front of the hit face (where a block would go).
    pub front: [i64; 3],
    /// Material of the targeted voxel.
    pub material: u16,
    /// Distance from the eye to the hit face.
    pub distance: f64,
    /// A block may be removed here.
    pub can_remove: bool,
    /// The selected block may be placed in front of the face.
    pub can_place: bool,
    /// Why not, when one of them is refused (empty when both are allowed).
    pub reason: String,
}

/// Hotbar, inventory and objective as the player sees them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GameView {
    /// `(item id, count)` per slot; id 0 is an empty slot. The first nine are the hotbar.
    pub slots: Vec<(u16, u32)>,
    /// Selected hotbar slot.
    pub selected: usize,
    /// Creative rules (unlimited materials) instead of survival.
    pub creative: bool,
    /// Objective title.
    pub objective_title: String,
    /// Objective progress text.
    pub objective_text: String,
    /// Objective finished.
    pub objective_done: bool,
    /// A short message for the player (empty when none).
    pub message: String,
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
    /// What the crosshair points at (within the interaction reach), if anything.
    fn target(&self) -> Option<TargetInfo> {
        None
    }
    /// Mines the targeted block under the game rules: `Ok(true)` done, `Ok(false)` nothing there,
    /// `Err(reason)` refused or failed (nothing changed).
    fn mine(&mut self) -> Result<bool, String> {
        Ok(false)
    }
    /// Places the selected block in front of the targeted face under the game rules.
    fn place(&mut self) -> Result<bool, String> {
        Ok(false)
    }
    /// Hotbar, inventory and objective.
    fn game_view(&self) -> GameView {
        GameView::default()
    }
    /// Selects a hotbar slot.
    fn select_slot(&mut self, _slot: usize) {}
    /// Moves the hotbar selection (mouse wheel).
    fn scroll_slot(&mut self, _delta: i32) {}
    /// Swaps two inventory slots.
    fn swap_slots(&mut self, _a: usize, _b: usize) {}
    /// Switches between creative and survival rules.
    fn set_creative(&mut self, _on: bool) {}
}
