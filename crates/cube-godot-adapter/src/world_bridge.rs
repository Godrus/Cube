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

/// Visible-face mesh of one chunk (experimental renderer). Four vertices and six indices per face;
/// flat arrays ready to become Godot mesh arrays. Faces whose neighbouring voxel is occupied are not
/// emitted. A neighbour in a chunk that is not loaded counts as air; below the world (`y < 0`) counts as
/// solid, so the unreachable bottom faces are never drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChunkMesh {
    /// `x, y, z` per vertex.
    pub positions: Vec<f32>,
    /// `x, y, z` per vertex.
    pub normals: Vec<f32>,
    /// `r, g, b, a` per vertex.
    pub colors: Vec<f32>,
    pub indices: Vec<u32>,
    pub faces: usize,
}

impl ChunkMesh {
    pub fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }

    /// Bytes of the arrays as handed to the renderer (positions, normals, colours, 32-bit indices).
    pub fn byte_size(&self) -> usize {
        (self.positions.len() + self.normals.len() + self.colors.len()) * 4 + self.indices.len() * 4
    }
}

const PAD_X: usize = 18;
const PAD_Z: usize = 18;
/// Padded grid height: `y = -1 ..= CHUNK_HEIGHT` (one cell of padding on both ends).
const PAD_Y: usize = CHUNK_HEIGHT as usize + 2;
const SOLID_BELOW_WORLD: u16 = u16::MAX;

/// Face table: outward normal, and the two edge vectors `u`, `v` with `u x v = normal`; `origin` is the
/// corner offset inside the voxel. Corners are emitted `p, p+v, p+u+v, p+u`, which is clockwise seen
/// from outside, the front-face winding of Godot.
type Face = ([i32; 3], [i32; 3], [i32; 3], [i32; 3]);
const FACES: [Face; 6] = [
    ([1, 0, 0], [0, 1, 0], [0, 0, 1], [1, 0, 0]),
    ([-1, 0, 0], [0, 0, 1], [0, 1, 0], [0, 0, 0]),
    ([0, 1, 0], [0, 0, 1], [1, 0, 0], [0, 1, 0]),
    ([0, -1, 0], [1, 0, 0], [0, 0, 1], [0, 0, 0]),
    ([0, 0, 1], [1, 0, 0], [0, 1, 0], [0, 0, 1]),
    ([0, 0, -1], [0, 1, 0], [1, 0, 0], [0, 0, 0]),
];

fn pad_index(x: i32, y: i32, z: i32) -> usize {
    ((y + 1) as usize * PAD_Z + (z + 1) as usize) * PAD_X + (x + 1) as usize
}

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

    /// Visible-face mesh of a Ready chunk in chunk-local-free world coordinates (same space as
    /// `chunk_instance_buffer`). Reads border neighbours from the neighbouring chunks, so a chunk must
    /// be rebuilt when a neighbour is loaded, unloaded or edited next to the border.
    pub fn chunk_mesh(&self, cx: i32, cz: i32) -> Option<ChunkMesh> {
        let coord = ChunkCoord::new(cx, cz);
        let cells = self.w.chunk_cells(coord)?;
        let (ox, oz) = (coord.origin_x(), coord.origin_z());
        let mut grid = vec![0u16; PAD_X * PAD_Y * PAD_Z];
        for z in -1..=16 {
            for x in -1..=16 {
                grid[pad_index(x, -1, z)] = SOLID_BELOW_WORLD;
            }
        }
        for &(p, m) in &cells {
            grid[pad_index(p.x - ox, p.y, p.z - oz)] = m;
        }
        for y in 0..CHUNK_HEIGHT {
            for i in 0..16 {
                for (lx, lz) in [(-1, i), (16, i), (i, -1), (i, 16)] {
                    let m = self
                        .w
                        .voxel(VoxelPos::new(ox + lx, y, oz + lz))
                        .unwrap_or(0);
                    grid[pad_index(lx, y, lz)] = m;
                }
            }
        }
        let mut mesh = ChunkMesh::default();
        for &(p, m) in &cells {
            let (lx, ly, lz) = (p.x - ox, p.y, p.z - oz);
            let col = material_color(m);
            for (n, u, v, o) in FACES {
                if grid[pad_index(lx + n[0], ly + n[1], lz + n[2])] != 0 {
                    continue;
                }
                let base = (mesh.positions.len() / 3) as u32;
                let p0 = [p.x + o[0], p.y + o[1], p.z + o[2]];
                for (a, b) in [(0, 0), (0, 1), (1, 1), (1, 0)] {
                    for k in 0..3 {
                        mesh.positions.push((p0[k] + a * u[k] + b * v[k]) as f32);
                        mesh.normals.push(n[k] as f32);
                    }
                    mesh.colors.extend_from_slice(&col);
                }
                mesh.indices.extend_from_slice(&[
                    base,
                    base + 1,
                    base + 2,
                    base,
                    base + 2,
                    base + 3,
                ]);
                mesh.faces += 1;
            }
        }
        Some(mesh)
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
