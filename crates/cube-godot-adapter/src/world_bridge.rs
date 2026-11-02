//! Pure-Rust bridge for the world-generation spike (no Godot types), tested without Godot.
//!
//! Wraps `cube_worldgen::WorldStreamer`. The adapter owns no terrain truth: voxels live in the
//! streamer's Geometry Core asset; the bridge only converts positions, pumps generation and prepares
//! flat arrays for the renderer.

use std::path::Path;

use cube_core::VoxelPos;
use cube_worldgen::generator::TerrainGenerator;
use cube_worldgen::state_v0;
use cube_worldgen::{
    ChunkCoord, ChunkState, PumpSummary, StateCounts, StreamConfig, UpdateSummary, WorldSeed,
    WorldStreamer, CHUNK_HEIGHT, MAT_PLACED,
};

pub const MAX_RADIUS: i32 = 8;
/// Floats per rendered cube: 12 for a 3x4 transform + 4 for the colour (Godot `MultiMesh` buffer layout).
pub const FLOATS_PER_INSTANCE: usize = 16;

pub struct WorldBridge {
    w: WorldStreamer,
    added: Vec<ChunkCoord>,
    removed: Vec<ChunkCoord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadSummary {
    pub saved_active: usize,
    pub restored_active: usize,
    pub missing: usize,
    pub edits_restored: usize,
}

fn floor_i32(v: f32) -> i32 {
    if v.is_nan() {
        0
    } else {
        v.floor().clamp(-2_000_000_000.0, 2_000_000_000.0) as i32
    }
}

pub fn material_color(m: u16) -> [f32; 4] {
    match m {
        1 => [0.35, 0.75, 0.25, 1.0],
        2 => [0.45, 0.30, 0.18, 1.0],
        3 => [0.50, 0.50, 0.55, 1.0],
        MAT_PLACED => [0.95, 0.55, 0.15, 1.0],
        _ => [1.0, 0.0, 1.0, 1.0],
    }
}

impl WorldBridge {
    pub fn new(seed: u64, radius: i32) -> Self {
        let cfg = StreamConfig {
            load_radius: radius.clamp(0, MAX_RADIUS),
            ..StreamConfig::default()
        };
        Self {
            w: WorldStreamer::new(WorldSeed(seed), Box::new(TerrainGenerator), cfg),
            added: Vec::new(),
            removed: Vec::new(),
        }
    }

    pub fn seed(&self) -> u64 {
        self.w.seed().0
    }

    pub fn radius(&self) -> i32 {
        self.w.config().load_radius
    }

    /// Sets the player/camera position (world units); queues chunks around it and unloads far ones.
    pub fn set_player(&mut self, x: f32, _y: f32, z: f32) -> UpdateSummary {
        self.w.set_center_world(floor_i32(x), floor_i32(z))
    }

    pub fn pump(&mut self, budget: usize) -> PumpSummary {
        self.w.pump(budget)
    }

    pub fn pump_all(&mut self) -> PumpSummary {
        self.w.pump_all()
    }

    fn drain_delta(&mut self) {
        let d = self.w.take_delta();
        self.added.extend(d.added);
        self.removed.extend(d.removed);
    }

    /// Chunks that became Ready since the last call (the client should add them).
    pub fn take_added(&mut self) -> Vec<[i32; 2]> {
        self.drain_delta();
        std::mem::take(&mut self.added)
            .into_iter()
            .map(|c| [c.x, c.z])
            .collect()
    }

    /// Chunks that were unloaded since the last call (the client should remove them).
    pub fn take_removed(&mut self) -> Vec<[i32; 2]> {
        self.drain_delta();
        std::mem::take(&mut self.removed)
            .into_iter()
            .map(|c| [c.x, c.z])
            .collect()
    }

    pub fn take_modified(&mut self) -> Vec<[i32; 2]> {
        self.w
            .take_modified()
            .into_iter()
            .map(|c| [c.x, c.z])
            .collect()
    }

    pub fn active_chunks(&self) -> Vec<[i32; 2]> {
        self.w
            .active_chunks()
            .into_iter()
            .map(|c| [c.x, c.z])
            .collect()
    }

    pub fn active_count(&self) -> usize {
        self.w.counts().ready
    }

    pub fn active_voxels(&self) -> usize {
        self.w.active_voxels()
    }

    pub fn counts(&self) -> StateCounts {
        self.w.counts()
    }

    pub fn queue_len(&self) -> usize {
        self.w.queue_len()
    }

    pub fn chunk_state(&self, cx: i32, cz: i32) -> ChunkState {
        self.w.state(ChunkCoord::new(cx, cz))
    }

    pub fn chunk_voxel_count(&self, cx: i32, cz: i32) -> Option<usize> {
        self.w.chunk_cells(ChunkCoord::new(cx, cz)).map(|v| v.len())
    }

    /// Flat `x, y, z, material` of every non-air voxel of a Ready chunk.
    pub fn chunk_voxels(&self, cx: i32, cz: i32) -> Option<Vec<i32>> {
        let cells = self.w.chunk_cells(ChunkCoord::new(cx, cz))?;
        let mut out = Vec::with_capacity(cells.len() * 4);
        for (p, m) in cells {
            out.extend_from_slice(&[p.x, p.y, p.z, m as i32]);
        }
        Some(out)
    }

    /// Ready-made `MultiMesh.buffer` content: per cube a row-major 3x4 identity-rotation transform at
    /// the voxel centre followed by an RGBA colour.
    pub fn chunk_instance_buffer(&self, cx: i32, cz: i32) -> Option<Vec<f32>> {
        let cells = self.w.chunk_cells(ChunkCoord::new(cx, cz))?;
        let mut out = Vec::with_capacity(cells.len() * FLOATS_PER_INSTANCE);
        for (p, m) in cells {
            let c = material_color(m);
            out.extend_from_slice(&[
                1.0,
                0.0,
                0.0,
                p.x as f32 + 0.5,
                0.0,
                1.0,
                0.0,
                p.y as f32 + 0.5,
                0.0,
                0.0,
                1.0,
                p.z as f32 + 0.5,
                c[0],
                c[1],
                c[2],
                c[3],
            ]);
        }
        Some(out)
    }

    pub fn ground_height(&self, x: i32, z: i32) -> i32 {
        TerrainGenerator::height_at(self.w.seed(), x, z)
    }

    /// Places a 3x3x3 block of `MAT_PLACED` centred on `(x, y, z)`; returns how many voxels changed.
    /// Voxels in chunks that are not Ready are skipped.
    pub fn edit_around(&mut self, x: i32, y: i32, z: i32) -> usize {
        let mut changed = 0;
        for dy in -1..=1 {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    let yy = (y + dy).clamp(0, CHUNK_HEIGHT - 1);
                    if let Ok(true) = self
                        .w
                        .set_voxel(VoxelPos::new(x + dx, yy, z + dz), MAT_PLACED)
                    {
                        changed += 1;
                    }
                }
            }
        }
        changed
    }

    pub fn edit_count(&self) -> usize {
        self.w.edit_count()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.w.save_state(path).map_err(|e| e.to_string())
    }

    /// Replaces the world with the saved one (same seed, config, edits, chunks regenerated).
    pub fn load(&mut self, path: &Path) -> Result<LoadSummary, String> {
        let (w, rep) =
            state_v0::load(path, Box::new(TerrainGenerator)).map_err(|e| e.to_string())?;
        self.w = w;
        // The client clears everything on load and then takes all active chunks as newly added.
        self.added = self.w.active_chunks();
        self.removed.clear();
        Ok(LoadSummary {
            saved_active: rep.saved_active,
            restored_active: rep.restored_active,
            missing: rep.missing.len(),
            edits_restored: rep.edits_restored,
        })
    }

    pub fn approx_brick_bytes(&self) -> usize {
        self.w.approx_brick_bytes()
    }
}
