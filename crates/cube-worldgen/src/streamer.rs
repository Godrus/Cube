//! WorldStreamer: keeps the chunks around a centre loaded in a `GeometryAsset` (Geometry Core).
//!
//! Synchronous by design: no threads, no scheduler (the async/job contracts are Draft). Generation
//! happens in `pump`, a bounded number of chunks per call.
//!
//! State machine (per chunk; "Unloaded" = no entry):
//!
//! ```text
//! Unloaded --set_center_world--> Queued --pump--> Generating --ok--> Ready
//!                                  |                  |                |
//!                                  |                  +--error--> Failed
//! Queued --(out of range)--> Unloaded                                  |
//! Ready  --(out of range)--> Unloaded     Failed --retry_failed--> Queued
//! Failed --(out of range)--> Unloaded
//! ```
//!
//! A chunk has at most one entry, so it is never queued or generated twice.

use std::collections::{HashMap, VecDeque};
use std::path::Path;

use cube_core::{
    AssetId, Brick, BrickCoord, BrickWrite, CoreError, ExpectedBrick, GeometryAsset, LayerId,
    LocalPos, Region, VoxelPos, WriteSet,
};

use crate::generator::{
    Chunk, ChunkCoord, ChunkGenerator, ChunkState, GenError, WorldSeed, BRICKS_PER_CHUNK,
    CHUNK_HEIGHT, CHUNK_SIZE,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamConfig {
    /// Chunks within this Chebyshev distance of the centre are wanted.
    pub load_radius: i32,
    /// Chunks farther than `load_radius + unload_margin` are unloaded (hysteresis against thrashing).
    pub unload_margin: i32,
    /// Maximum number of chunks waiting for generation; extra wanted chunks are deferred to a later update.
    pub max_queue: usize,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            load_radius: 1,
            unload_margin: 1,
            max_queue: 64,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoxelEdit {
    pub pos: VoxelPos,
    pub material: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldError {
    ChunkNotReady(ChunkCoord),
    OutOfBounds,
    Core(CoreError),
}

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorldError::ChunkNotReady(c) => {
                write!(f, "WORLD-0001: chunk ({}, {}) is not ready", c.x, c.z)
            }
            WorldError::OutOfBounds => write!(f, "WORLD-0002: position outside chunk height"),
            WorldError::Core(e) => write!(f, "WORLD-0003: {e}"),
        }
    }
}

impl std::error::Error for WorldError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UpdateSummary {
    pub queued: usize,
    pub unloaded: usize,
    pub dropped_from_queue: usize,
    /// Wanted chunks that did not fit into the bounded queue; they are picked up by a later update.
    pub deferred: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PumpSummary {
    pub generated: usize,
    pub failed: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StreamDelta {
    pub added: Vec<ChunkCoord>,
    pub removed: Vec<ChunkCoord>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StateCounts {
    pub queued: usize,
    pub generating: usize,
    pub ready: usize,
    pub failed: usize,
}

struct Entry {
    state: ChunkState,
    error: Option<GenError>,
}

pub struct WorldStreamer {
    seed: WorldSeed,
    generator: Box<dyn ChunkGenerator>,
    cfg: StreamConfig,
    asset: GeometryAsset,
    layer: LayerId,
    entries: HashMap<ChunkCoord, Entry>,
    queue: VecDeque<ChunkCoord>,
    center: Option<ChunkCoord>,
    /// Player edits per chunk; survive unloading and are re-applied when the chunk is generated again.
    pub(crate) edits: HashMap<ChunkCoord, Vec<VoxelEdit>>,
    added: Vec<ChunkCoord>,
    removed: Vec<ChunkCoord>,
    modified: Vec<ChunkCoord>,
    trace: Option<Vec<(ChunkCoord, ChunkState)>>,
}

impl WorldStreamer {
    pub fn new(seed: WorldSeed, generator: Box<dyn ChunkGenerator>, cfg: StreamConfig) -> Self {
        let mut asset = GeometryAsset::new(AssetId(1));
        let layer = asset.add_voxel_layer(1.0);
        Self {
            seed,
            generator,
            cfg,
            asset,
            layer,
            entries: HashMap::new(),
            queue: VecDeque::new(),
            center: None,
            edits: HashMap::new(),
            added: Vec::new(),
            removed: Vec::new(),
            modified: Vec::new(),
            trace: None,
        }
    }

    pub fn seed(&self) -> WorldSeed {
        self.seed
    }

    pub fn config(&self) -> StreamConfig {
        self.cfg
    }

    pub fn center(&self) -> Option<ChunkCoord> {
        self.center
    }

    pub fn asset(&self) -> &GeometryAsset {
        &self.asset
    }

    pub(crate) fn asset_mut(&mut self) -> &mut GeometryAsset {
        &mut self.asset
    }

    pub fn layer(&self) -> LayerId {
        self.layer
    }

    /// Records every state change (for tests of the state machine).
    pub fn enable_trace(&mut self) {
        self.trace = Some(Vec::new());
    }

    pub fn trace(&self) -> &[(ChunkCoord, ChunkState)] {
        self.trace.as_deref().unwrap_or(&[])
    }

    pub fn state(&self, c: ChunkCoord) -> ChunkState {
        self.entries
            .get(&c)
            .map_or(ChunkState::Unloaded, |e| e.state)
    }

    pub fn failure(&self, c: ChunkCoord) -> Option<&GenError> {
        self.entries.get(&c).and_then(|e| e.error.as_ref())
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn counts(&self) -> StateCounts {
        let mut n = StateCounts::default();
        for e in self.entries.values() {
            match e.state {
                ChunkState::Queued => n.queued += 1,
                ChunkState::Generating => n.generating += 1,
                ChunkState::Ready => n.ready += 1,
                ChunkState::Failed => n.failed += 1,
                ChunkState::Unloaded => {}
            }
        }
        n
    }

    /// Ready chunks, sorted.
    pub fn active_chunks(&self) -> Vec<ChunkCoord> {
        let mut v: Vec<_> = self
            .entries
            .iter()
            .filter(|(_, e)| e.state == ChunkState::Ready)
            .map(|(c, _)| *c)
            .collect();
        v.sort();
        v
    }

    /// Total non-air voxels in Ready chunks.
    pub fn active_voxels(&self) -> usize {
        self.active_chunks()
            .iter()
            .map(|c| self.chunk_cells(*c).map_or(0, |v| v.len()))
            .sum()
    }

    fn set_state(&mut self, c: ChunkCoord, state: ChunkState) {
        if let Some(t) = &mut self.trace {
            t.push((c, state));
        }
        if state == ChunkState::Unloaded {
            self.entries.remove(&c);
        } else {
            self.entries
                .entry(c)
                .or_insert(Entry { state, error: None })
                .state = state;
        }
    }

    /// Moves the streaming centre to the chunk containing world column `(wx, wz)`, unloads chunks that
    /// are too far and queues the missing ones (nearest first, bounded by `max_queue`).
    pub fn set_center_world(&mut self, wx: i32, wz: i32) -> UpdateSummary {
        let center = ChunkCoord::from_world(wx, wz);
        self.center = Some(center);
        let mut sum = UpdateSummary::default();
        let keep = self.cfg.load_radius + self.cfg.unload_margin;

        let far: Vec<ChunkCoord> = self
            .entries
            .keys()
            .copied()
            .filter(|c| c.distance(center) > keep)
            .collect();
        for c in far {
            match self.state(c) {
                ChunkState::Queued => {
                    self.queue.retain(|q| *q != c);
                    self.set_state(c, ChunkState::Unloaded);
                    sum.dropped_from_queue += 1;
                }
                ChunkState::Ready => {
                    self.unload_ready(c);
                    sum.unloaded += 1;
                }
                ChunkState::Failed => self.set_state(c, ChunkState::Unloaded),
                ChunkState::Generating | ChunkState::Unloaded => {}
            }
        }

        let r = self.cfg.load_radius;
        let mut wanted = Vec::new();
        for dz in -r..=r {
            for dx in -r..=r {
                let c = ChunkCoord::new(center.x + dx, center.z + dz);
                if self.state(c) == ChunkState::Unloaded {
                    wanted.push(c);
                }
            }
        }
        wanted.sort_by_key(|c| (c.distance(center), c.x, c.z));
        for c in wanted {
            if self.queue.len() >= self.cfg.max_queue {
                sum.deferred += 1;
                continue;
            }
            self.set_state(c, ChunkState::Queued);
            self.queue.push_back(c);
            sum.queued += 1;
        }
        sum
    }

    /// Queues a specific chunk regardless of the radius (used when restoring a saved state, whose active
    /// set can include chunks kept by the unload margin). No-op unless the chunk is Unloaded.
    pub(crate) fn queue_explicit(&mut self, c: ChunkCoord) {
        if self.state(c) != ChunkState::Unloaded {
            return;
        }
        if self.queue.len() >= self.cfg.max_queue {
            self.pump_all();
        }
        self.set_state(c, ChunkState::Queued);
        self.queue.push_back(c);
    }

    /// Generates up to `budget` queued chunks. A failing chunk becomes `Failed`; others are unaffected.
    pub fn pump(&mut self, budget: usize) -> PumpSummary {
        let mut sum = PumpSummary::default();
        for _ in 0..budget {
            let Some(c) = self.queue.pop_front() else {
                break;
            };
            if self.state(c) != ChunkState::Queued {
                continue;
            }
            self.set_state(c, ChunkState::Generating);
            let result = self.generator.generate(self.seed, c).and_then(|chunk| {
                self.install(chunk)
                    .map_err(|e| GenError::Commit(e.to_string()))
            });
            match result {
                Ok(()) => {
                    self.set_state(c, ChunkState::Ready);
                    self.added.push(c);
                    sum.generated += 1;
                }
                Err(e) => {
                    self.set_state(c, ChunkState::Failed);
                    if let Some(en) = self.entries.get_mut(&c) {
                        en.error = Some(e);
                    }
                    sum.failed += 1;
                }
            }
        }
        sum
    }

    /// Pumps until the queue is empty.
    pub fn pump_all(&mut self) -> PumpSummary {
        let mut total = PumpSummary::default();
        while !self.queue.is_empty() {
            let s = self.pump(self.queue.len());
            total.generated += s.generated;
            total.failed += s.failed;
        }
        total
    }

    /// Re-queues `Failed` chunks that are still within the load radius. Returns how many.
    pub fn retry_failed(&mut self) -> usize {
        let Some(center) = self.center else { return 0 };
        let failed: Vec<ChunkCoord> = self
            .entries
            .iter()
            .filter(|(c, e)| {
                e.state == ChunkState::Failed && c.distance(center) <= self.cfg.load_radius
            })
            .map(|(c, _)| *c)
            .collect();
        let mut n = 0;
        for c in failed {
            if self.queue.len() >= self.cfg.max_queue {
                break;
            }
            self.entries.remove(&c);
            self.set_state(c, ChunkState::Queued);
            self.queue.push_back(c);
            n += 1;
        }
        n
    }

    /// Chunks that became Ready / were unloaded since the last call.
    pub fn take_delta(&mut self) -> StreamDelta {
        StreamDelta {
            added: std::mem::take(&mut self.added),
            removed: std::mem::take(&mut self.removed),
        }
    }

    /// Chunks whose voxels changed through `set_voxel` since the last call.
    pub fn take_modified(&mut self) -> Vec<ChunkCoord> {
        std::mem::take(&mut self.modified)
    }

    fn brick_coord(c: ChunkCoord, bi: usize) -> BrickCoord {
        BrickCoord::new(c.x, bi as i32, c.z)
    }

    /// Writes a generated chunk (plus recorded edits) into Geometry Core in one commit.
    fn install(&mut self, mut chunk: Chunk) -> Result<(), CoreError> {
        if let Some(edits) = self.edits.get(&chunk.coord) {
            for e in edits {
                let ly = e.pos.y;
                let bi = (ly / 16) as usize;
                let local = LocalPos {
                    x: e.pos.x.rem_euclid(CHUNK_SIZE) as u8,
                    y: (ly % 16) as u8,
                    z: e.pos.z.rem_euclid(CHUNK_SIZE) as u8,
                };
                let b = chunk.bricks[bi].get_or_insert_with(|| Brick::constant(0));
                b.set(local, e.material);
            }
            for b in chunk.bricks.iter_mut() {
                if let Some(br) = b {
                    br.normalize();
                    if br.is_empty() {
                        *b = None;
                    }
                }
            }
        }
        let mut writes = Vec::new();
        for (bi, b) in chunk.bricks.into_iter().enumerate() {
            if let Some(b) = b {
                let coord = Self::brick_coord(chunk.coord, bi);
                writes.push(BrickWrite {
                    coord,
                    expected: ExpectedBrick::Absent,
                    new: Some(b),
                    changed: Region::of_brick(coord),
                });
            }
        }
        self.asset
            .commit(WriteSet {
                layer: self.layer,
                operation: None,
                writes,
            })
            .map(|_| ())
    }

    fn unload_ready(&mut self, c: ChunkCoord) {
        let mut writes = Vec::new();
        for bi in 0..BRICKS_PER_CHUNK {
            let coord = Self::brick_coord(c, bi);
            if let Ok(ExpectedBrick::Revision(r)) = self.asset.expected_state(self.layer, coord) {
                writes.push(BrickWrite {
                    coord,
                    expected: ExpectedBrick::Revision(r),
                    new: None,
                    changed: Region::of_brick(coord),
                });
            }
        }
        // Removing bricks that exist cannot conflict; an empty write set is a no-op.
        let _ = self.asset.commit(WriteSet {
            layer: self.layer,
            operation: None,
            writes,
        });
        self.set_state(c, ChunkState::Unloaded);
        self.removed.push(c);
    }

    /// Explicitly unloads a Ready chunk (also used by tests). Returns false if it was not Ready.
    pub fn unload(&mut self, c: ChunkCoord) -> bool {
        if self.state(c) == ChunkState::Ready {
            self.unload_ready(c);
            true
        } else {
            false
        }
    }

    /// Non-air voxels of a Ready chunk in world coordinates, with materials (deterministic order).
    pub fn chunk_cells(&self, c: ChunkCoord) -> Option<Vec<(VoxelPos, u16)>> {
        if self.state(c) != ChunkState::Ready {
            return None;
        }
        let layer = self.asset.layer(self.layer).ok()?;
        let mut out = Vec::new();
        for bi in 0..BRICKS_PER_CHUNK {
            let coord = Self::brick_coord(c, bi);
            if let Some(b) = layer.storage().get(coord) {
                let mut cells: Vec<_> = b
                    .occupied()
                    .into_iter()
                    .map(|(l, m)| (coord.join(l), m))
                    .collect();
                cells.sort_by_key(|(p, _)| (p.y, p.z, p.x));
                out.extend(cells);
            }
        }
        Some(out)
    }

    /// Stable hash (see `generator::stable_hash_cells`) of what Geometry Core currently holds for a
    /// Ready chunk. Equals `Chunk::stable_hash` of the generated chunk while the chunk has no edits.
    pub fn chunk_stable_hash(&self, c: ChunkCoord) -> Option<u64> {
        let mut cells: Vec<_> = self
            .chunk_cells(c)?
            .into_iter()
            .map(|(p, m)| (p.x, p.y, p.z, m))
            .collect();
        Some(crate::generator::stable_hash_cells(
            self.seed,
            crate::generator::GENERATOR_VERSION,
            c,
            &mut cells,
        ))
    }

    /// Material at a world voxel of a Ready chunk.
    pub fn voxel(&self, p: VoxelPos) -> Option<u16> {
        let c = ChunkCoord::from_world(p.x, p.z);
        if self.state(c) != ChunkState::Ready || !(0..CHUNK_HEIGHT).contains(&p.y) {
            return None;
        }
        let (bc, l) = p.split();
        let layer = self.asset.layer(self.layer).ok()?;
        Some(layer.storage().get(bc).map_or(0, |b| b.get(l)))
    }

    /// Edits one voxel of a Ready chunk through the Core commit primitive and records the edit so it
    /// survives unloading. No undo for world edits in this spike.
    pub fn set_voxel(&mut self, p: VoxelPos, material: u16) -> Result<bool, WorldError> {
        if !(0..CHUNK_HEIGHT).contains(&p.y) {
            return Err(WorldError::OutOfBounds);
        }
        let c = ChunkCoord::from_world(p.x, p.z);
        if self.state(c) != ChunkState::Ready {
            return Err(WorldError::ChunkNotReady(c));
        }
        let (bc, l) = p.split();
        let existing = self
            .asset
            .layer(self.layer)
            .map_err(WorldError::Core)?
            .storage()
            .get(bc)
            .cloned();
        let mut work = existing
            .as_deref()
            .cloned()
            .unwrap_or_else(|| Brick::constant(0));
        if !work.set(l, material) {
            return Ok(false);
        }
        work.normalize();
        let expected = match &existing {
            Some(b) => ExpectedBrick::Revision(b.revision()),
            None => ExpectedBrick::Absent,
        };
        let new = if work.is_empty() { None } else { Some(work) };
        let changed = Region::new(p, VoxelPos::new(p.x + 1, p.y + 1, p.z + 1));
        self.asset
            .commit(WriteSet {
                layer: self.layer,
                operation: None,
                writes: vec![BrickWrite {
                    coord: bc,
                    expected,
                    new,
                    changed,
                }],
            })
            .map_err(WorldError::Core)?;
        let list = self.edits.entry(c).or_default();
        list.retain(|e| e.pos != p);
        list.push(VoxelEdit { pos: p, material });
        if !self.modified.contains(&c) {
            self.modified.push(c);
        }
        Ok(true)
    }

    pub fn edit_count(&self) -> usize {
        self.edits.values().map(|v| v.len()).sum()
    }

    /// Approximate bytes held by loaded bricks (dense bricks are 8 KiB, constant bricks tiny).
    pub fn approx_brick_bytes(&self) -> usize {
        self.asset.layer(self.layer).map_or(0, |l| {
            l.storage()
                .bricks_sorted()
                .iter()
                .map(|(_, b)| match b.data() {
                    cube_core::BrickData::Constant(_) => 16,
                    cube_core::BrickData::Dense(_) => 8192,
                })
                .sum()
        })
    }

    /// Experimental `world-state-v0` save; see `state_v0`.
    pub fn save_state(&self, path: &Path) -> Result<(), crate::state_v0::StateError> {
        crate::state_v0::save(self, path)
    }
}
