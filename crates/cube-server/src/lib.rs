//! The in-process local host of one world (first world; DEC-092). Not the networked server
//! (PS-13), not a child process: a library that the application process calls.
//!
//! It owns everything authoritative about a local world and nothing about how it is shown:
//! the Core asset and its handles, Orchestration with the voxel backend, the world record, the
//! manifest and the file journal, the generator and the Processing executor (faces provider).
//!
//! * the **untouched base** is never saved: it is regenerated from the record's seed and
//!   generator version by the system-commit port, which never overwrites a loaded cell. A
//!   **static initial area** is loaded when the world opens, and [`LocalHost::ensure_loaded`]
//!   streams more bricks in around the player (never unloading any: a brick's revision must be
//!   the same after a restart, or the journal's expectations would not hold);
//! * every **user edit** is a journalled operation (acknowledged = fsynced); opening a world
//!   regenerates the area, replays the confirmed edits and rebuilds the retry table;
//! * viewers (the client) read through [`LocalHost::reader`] and [`LocalHost::executor`], hear
//!   about commits through [`LocalHost::add_dirty_listener`] and ask for changes through
//!   [`LocalHost::submit_edit`]. They never hold a handle.
//!
//! Operation ids use one scope and strictly increasing sequences that continue after a restart
//! (requirement RS-1), so old retries are always answered or refused, never applied twice.

use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use cube_backend_voxel::{FacesProvider, VoxelBackend, add, remove};
use std::collections::{HashMap, HashSet};

use cube_core::{
    AssetHost, AssetId, BrickCoord, DirtyRegion, GeometryAsset, GeometryTransform, Layer,
    LayerData, LayerId, LayerKind, OperationId, Region, SharedAsset, SystemCommit, VoxelPos,
};
use cube_editing_api::{
    ActorId, AuthorizationContext, Capability, EditMode, EditingOperation, EditingResult,
    EditingService, JournalPhase, JournalSeq, OperationJournal, Origin, Scope, SessionId,
    TransactionSpec, kinds,
};
use cube_orchestration::{
    BackendRegistry, CapabilityAuthorizer, CommitPath, Config, DirtySink, OrchestrationService,
};
use cube_persistence::{
    DirStorage, Limits, WorldRecord, create_world_with_record, open_world_with_record,
    replay_confirmed,
};
use cube_processing::{Executor, ExecutorConfig, ManualClock, ProviderRegistry};
use cube_worldgen::{GenError, generate_brick, registry};

/// The generator version that new worlds are created with.
pub fn current_generator_version() -> cube_core::GeneratorVersion {
    cube_worldgen::current_version()
}

/// The one asset of a local world.
pub const ASSET: u64 = 1;
/// The voxel layer.
pub const VOXEL_LAYER: u64 = 1;
/// The world is finite: bricks further than this many bricks from the origin column are never
/// generated (the player meets an edge, not an endless world and an endless memory bill).
pub const WORLD_LIMIT_BRICKS: i64 = 48;
/// The operation scope of the local player.
pub const PLAYER_SCOPE: u64 = 1;
const EDIT: Capability = Capability(1);

/// A static, bounded area of bricks that the host loads: the cube of bricks within `radius` of
/// `centre`. This is not streaming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticArea {
    /// Centre brick.
    pub centre: BrickCoord,
    /// Radius in bricks (the area has `(2r+1)^3` bricks).
    pub radius: i64,
}

impl StaticArea {
    /// The area around the surface of the column `(0, 0)` of the record's world.
    pub fn around_origin(record: &WorldRecord, radius: i64) -> Self {
        let h =
            cube_worldgen::surface_height(record.seed, record.generator_version, 0, 0).unwrap_or(0);
        Self {
            centre: BrickCoord::new(0, h.div_euclid(16), 0),
            radius,
        }
    }

    fn bricks(&self) -> Vec<BrickCoord> {
        let r = self.radius;
        let mut v = Vec::new();
        for z in -r..=r {
            for y in -r..=r {
                for x in -r..=r {
                    v.push(BrickCoord::new(
                        self.centre.x + x,
                        self.centre.y + y,
                        self.centre.z + z,
                    ));
                }
            }
        }
        v
    }
}

/// A request to change one voxel (the same shape the client produces; the client and this
/// crate do not depend on each other).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditRequest {
    /// Put the material into the (empty) voxel.
    Place {
        /// The voxel.
        voxel: [i64; 3],
        /// Material (non-zero).
        material: u16,
    },
    /// Clear the voxel.
    Remove {
        /// The voxel.
        voxel: [i64; 3],
    },
}

/// Why a host call failed.
#[derive(Debug)]
#[non_exhaustive]
pub enum HostError {
    /// Persistence refused (damaged, missing, unavailable generator, ...).
    Persistence(String),
    /// The generator failed.
    Generator(GenError),
    /// Core or Processing setup refused.
    Setup(String),
    /// Orchestration refused the edit.
    Edit(String),
    /// Replaying the journal failed for these operations.
    Replay(Vec<(OperationId, String)>),
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for HostError {}

/// Passes commit notifications to the executor and to every registered listener.
struct Fanout {
    executor: Arc<Executor>,
    listeners: Mutex<Vec<Arc<dyn DirtySink>>>,
}

impl DirtySink for Fanout {
    fn dirty(&self, regions: &[DirtyRegion]) {
        self.executor.dirty(regions);
        let ls: Vec<Arc<dyn DirtySink>> = lock(&self.listeners).clone();
        for l in ls {
            l.dirty(regions);
        }
    }
}

/// One local world.
pub struct LocalHost {
    record: WorldRecord,
    area: StaticArea,
    reader: SharedAsset,
    svc: Arc<OrchestrationService>,
    executor: Arc<Executor>,
    fanout: Arc<Fanout>,
    next_seq: Mutex<u64>,
    system: SystemCommit,
    streamed: Mutex<Streamed>,
    /// Sequences of player operations whose last journal record was `Confirmed` when the world
    /// was opened (used to reconcile state that lives outside the journal, see
    /// [`LocalHost::edit_confirmed_at_open`]).
    confirmed_at_open: HashSet<u64>,
    /// The writer lock of the world directory, held until the host is dropped (M1-A5).
    _lock: cube_persistence::WorldLock,
}

/// Bricks resident in Core (generated or edited), the bricks the player has edited, and the next
/// system operation number.
struct Streamed {
    loaded: HashSet<BrickCoord>,
    /// Bricks with user edits: they are never unloaded (their content is not regenerable).
    edited: HashSet<BrickCoord>,
    next_op: u64,
    loads: u64,
    evictions: u64,
}

/// Counters of the streaming state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamStats {
    /// Bricks resident in Core now.
    pub resident: usize,
    /// Of those, bricks with user edits (pinned).
    pub edited: usize,
    /// Bricks generated or reloaded since the host opened.
    pub loads: u64,
    /// Bricks unloaded since the host opened.
    pub evictions: u64,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

fn persistence(e: impl std::fmt::Display) -> HostError {
    HostError::Persistence(e.to_string())
}

fn setup(e: impl std::fmt::Debug) -> HostError {
    HostError::Setup(format!("{e:?}"))
}

fn runnable(v: cube_core::GeneratorVersion) -> bool {
    registry().iter().any(|i| i.version == v)
}

impl LocalHost {
    /// Creates a new world in `dir` (which must not hold one) and opens it.
    pub fn create(dir: &Path, record: WorldRecord, area: StaticArea) -> Result<Self, HostError> {
        let lock = cube_persistence::WorldLock::acquire(dir).map_err(persistence)?;
        let mut st = DirStorage::open(dir).map_err(persistence)?;
        create_world_with_record(&mut st, &record, &Limits::default()).map_err(persistence)?;
        Self::open_locked(dir, area, lock)
    }

    /// Opens the world in `dir`: reads the record, regenerates the base area, replays the
    /// confirmed edits and rebuilds the retry table. The generator version of the record is
    /// never substituted.
    ///
    /// The world is opened for writing by one host at a time: a second `open` of a directory that
    /// another host (in this or another process) holds fails at once with a persistence error.
    pub fn open(dir: &Path, area: StaticArea) -> Result<Self, HostError> {
        let lock = cube_persistence::WorldLock::acquire(dir).map_err(persistence)?;
        Self::open_locked(dir, area, lock)
    }

    fn open_locked(
        dir: &Path,
        area: StaticArea,
        lock: cube_persistence::WorldLock,
    ) -> Result<Self, HostError> {
        let limits = Limits::default();
        let (record, world) = open_world_with_record(
            DirStorage::open(dir).map_err(persistence)?,
            &limits,
            &runnable,
        )
        .map_err(persistence)?;
        let entries = world
            .journal
            .read_from(JournalSeq(0), usize::MAX)
            .map_err(persistence)?;
        let floors = world.manifest.retry_floors.clone();
        let journal: Arc<dyn OperationJournal> = Arc::new(world.journal);
        let snapshot = world.snapshot;

        let layers = vec![Layer::new(
            LayerId::new(VOXEL_LAYER),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Voxel),
        )];
        let asset_host = match &snapshot {
            Some(view) => AssetHost::from_view(view).map_err(setup)?,
            None => AssetHost::new(GeometryAsset::new(AssetId::new(ASSET), layers).map_err(setup)?),
        };
        let reader = asset_host.reader();
        let mut providers = ProviderRegistry::new();
        providers.register(Arc::new(FacesProvider)).map_err(setup)?;
        let executor = Arc::new(Executor::new(
            reader.clone(),
            providers.freeze().map_err(setup)?,
            ExecutorConfig::default(),
            Arc::new(ManualClock::new(0)),
        ));
        let fanout = Arc::new(Fanout {
            executor: executor.clone(),
            listeners: Mutex::new(Vec::new()),
        });
        let handles = asset_host.into_handles();
        let mut backends = BackendRegistry::new();
        backends.register(VoxelBackend::arc()).map_err(setup)?;
        let svc = Arc::new(OrchestrationService::new(Config {
            path: CommitPath::new(handles.user),
            registry: backends,
            authorizer: Arc::new(CapabilityAuthorizer { edit: EDIT }),
            journal: Some(journal),
            dirty: Some(fanout.clone()),
            observers: vec![],
            retry_capacity: 4096,
            batch_encoder: Some(Arc::new(cube_persistence::BatchCodec)),
        }));

        // The untouched base of the static area, then the user's edits on top of it.
        let mut streamed = Streamed {
            loaded: HashSet::new(),
            edited: HashSet::new(),
            next_op: 1,
            loads: 0,
            evictions: 0,
        };
        if snapshot.is_none() {
            let bricks = area.bricks();
            materialise(&handles.system, &record, &bricks, &mut streamed)?;
            // Bricks that earlier sessions edited may lie outside the initial area: their base must
            // exist before their edits are replayed.
            // A brick may have been unloaded and loaded again during the session that wrote the
            // journal; every load raises its revision by one, and an edit records the revision it
            // expected. So the base is brought to the first recorded expectation of each cell
            // (by the same unload and load steps, never by resetting a counter) before replay.
            let mut first_seen: HashSet<BrickCoord> = HashSet::new();
            for e in entries.iter().filter(|e| !e.payload.is_empty()) {
                let Ok(batch) = cube_persistence::decode_batch(&e.payload, &limits) else {
                    continue;
                };
                for set in &batch.sets {
                    for w in &set.writes {
                        streamed.edited.insert(w.brick);
                        if !first_seen.insert(w.brick) {
                            continue;
                        }
                        materialise(&handles.system, &record, &[w.brick], &mut streamed)?;
                        if let cube_core::ExpectedRevision::At(want) = w.expected {
                            bring_to_revision(
                                &handles.system,
                                &reader,
                                &record,
                                w.brick,
                                want,
                                &mut streamed,
                            )?;
                        }
                    }
                }
            }
        }
        let report = replay_confirmed(&entries, &handles.recovery, &limits);
        if !report.failed.is_empty() {
            return Err(HostError::Replay(report.failed));
        }
        svc.restore_retry_floors(&floors);
        svc.recover_idempotency().map_err(setup)?;
        let next_seq = entries
            .iter()
            .flat_map(|e| e.members.iter().map(|(id, _)| *id).chain([e.operation]))
            .filter(|id| id.scope() == PLAYER_SCOPE)
            .map(|id| id.sequence())
            .chain(
                floors
                    .iter()
                    .filter(|(s, _)| *s == PLAYER_SCOPE)
                    .map(|(_, q)| *q),
            )
            .max()
            .map_or(1, |m| m + 1);
        // The last journal phase of every player operation: only a `Confirmed` one changed the
        // world (an `Intent` without a decision is not replayed, see `replay_confirmed`).
        let mut last: HashMap<OperationId, JournalPhase> = HashMap::new();
        for e in &entries {
            last.insert(e.operation, e.phase);
        }
        let mut confirmed_at_open: HashSet<u64> = HashSet::new();
        for e in &entries {
            if e.phase == JournalPhase::Confirmed && last.get(&e.operation) == Some(&e.phase) {
                confirmed_at_open.extend(
                    e.members
                        .iter()
                        .map(|(id, _)| *id)
                        .chain([e.operation])
                        .filter(|id| id.scope() == PLAYER_SCOPE)
                        .map(|id| id.sequence()),
                );
            }
        }
        Ok(Self {
            record,
            area,
            reader,
            svc,
            executor,
            fanout,
            next_seq: Mutex::new(next_seq),
            system: handles.system,
            streamed: Mutex::new(streamed),
            confirmed_at_open,
            _lock: lock,
        })
    }

    /// `true` if the player operation with this sequence was confirmed in the journal when this
    /// world was opened, i.e. the change is part of the restored world. A game that keeps state
    /// outside the journal (the inventory) writes its intent first and asks this after a restart
    /// to learn whether the world change happened. An operation that was only started (an
    /// `Intent` without decision), aborted or never begun is not confirmed. Operations committed
    /// by this very session are not listed: ask after a restart only.
    pub fn edit_confirmed_at_open(&self, seq: u64) -> bool {
        self.confirmed_at_open.contains(&seq)
    }

    /// Takes the next player operation sequence for an edit that will be submitted with
    /// [`LocalHost::submit_edit_reserved`]. A sequence that is never used leaves a gap, which
    /// the retry rules allow (only strictly increasing is required).
    pub fn reserve_edit_seq(&self) -> u64 {
        let mut n = lock(&self.next_seq);
        let s = *n;
        *n += 1;
        s
    }

    /// Generates (into Core, by the system-commit port) the bricks of the cube of `radius` around
    /// `centre` that are not loaded yet, nearest first, at most `budget` of them. Returns how
    /// many bricks of that cube are still missing. Bricks beyond [`WORLD_LIMIT_BRICKS`]
    /// horizontally are never generated. Nothing is ever unloaded.
    pub fn ensure_loaded(
        &self,
        centre: BrickCoord,
        radius: i64,
        budget: usize,
    ) -> Result<usize, HostError> {
        let mut wanted: Vec<BrickCoord> = Vec::new();
        {
            let st = lock(&self.streamed);
            for dz in -radius..=radius {
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let b = BrickCoord::new(centre.x + dx, centre.y + dy, centre.z + dz);
                        if b.x.abs() <= WORLD_LIMIT_BRICKS
                            && b.z.abs() <= WORLD_LIMIT_BRICKS
                            && !st.loaded.contains(&b)
                        {
                            wanted.push(b);
                        }
                    }
                }
            }
        }
        let dist = |b: &BrickCoord| {
            let d = [b.x - centre.x, b.y - centre.y, b.z - centre.z];
            d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
        };
        wanted.sort_by_key(|b| (dist(b), b.x, b.y, b.z));
        let missing = wanted.len();
        let now: Vec<BrickCoord> = wanted.into_iter().take(budget).collect();
        let done = now.len();
        let mut st = lock(&self.streamed);
        materialise(&self.system, &self.record, &now, &mut st)?;
        Ok(missing - done)
    }

    /// Number of bricks resident in Core now (diagnostics).
    pub fn loaded_bricks(&self) -> usize {
        lock(&self.streamed).loaded.len()
    }

    /// Streaming counters.
    pub fn stream_stats(&self) -> StreamStats {
        let st = lock(&self.streamed);
        StreamStats {
            resident: st.loaded.len(),
            edited: st.edited.len(),
            loads: st.loads,
            evictions: st.evictions,
        }
    }

    /// Unloads bricks the player is far from. A resident brick is a candidate when it has no user
    /// edit and is further than `evict_radius` bricks from `centre`; if more than `max_resident`
    /// bricks are resident, the farthest candidates beyond `protect_radius` go too. Bricks with
    /// user edits and bricks within `protect_radius` are never unloaded. The derived data of
    /// unloaded bricks is dropped from the Processing cache. Returns how many bricks were
    /// unloaded. Unloaded base bricks are regenerated, identical, when the player comes back.
    pub fn evict_far(
        &self,
        centre: BrickCoord,
        evict_radius: i64,
        protect_radius: i64,
        max_resident: usize,
    ) -> Result<usize, HostError> {
        let dist = |b: &BrickCoord| {
            (b.x - centre.x)
                .abs()
                .max((b.y - centre.y).abs())
                .max((b.z - centre.z).abs())
        };
        let mut st = lock(&self.streamed);
        let mut victims: Vec<BrickCoord> = st
            .loaded
            .iter()
            .filter(|b| !st.edited.contains(b) && dist(b) > evict_radius)
            .copied()
            .collect();
        let over = st.loaded.len().saturating_sub(victims.len()) > max_resident;
        if over {
            let mut more: Vec<BrickCoord> = st
                .loaded
                .iter()
                .filter(|b| {
                    !st.edited.contains(b) && dist(b) > protect_radius && dist(b) <= evict_radius
                })
                .copied()
                .collect();
            more.sort_by_key(|b| (std::cmp::Reverse(dist(b)), b.x, b.y, b.z));
            let excess = st.loaded.len() - victims.len() - max_resident;
            victims.extend(more.into_iter().take(excess));
        }
        // One step for the whole list: the layer store is copied once, not once per brick.
        self.system
            .evict_many(LayerId::new(VOXEL_LAYER), 0, &victims)
            .map_err(setup)?;
        let mut targets = Vec::with_capacity(victims.len());
        for b in &victims {
            st.loaded.remove(b);
            st.evictions += 1;
            if let Ok(key) = cube_core::CellKey::new(LayerId::new(VOXEL_LAYER), 0, *b) {
                targets.push(cube_processing::DerivedTarget {
                    kind: cube_backend_voxel::FACES_KIND,
                    key,
                });
            }
        }
        drop(st);
        self.executor.forget(&targets);
        Ok(victims.len())
    }

    /// Height of the ground at column `(x, z)` in this world (voxels), for placing a camera.
    pub fn surface_height(&self, x: i64, z: i64) -> i64 {
        cube_worldgen::surface_height(self.record.seed, self.record.generator_version, x, z)
            .unwrap_or(0)
    }

    /// The name of the biome of the voxel column `(x, z)` ("" for a generator version without
    /// biomes in its registry entry).
    pub fn biome_name(&self, x: i64, z: i64) -> &'static str {
        match cube_worldgen::biome_at(self.record.seed, self.record.generator_version, x, z) {
            Ok(cube_worldgen::v2::Biome::Plains) => "Plains",
            Ok(cube_worldgen::v2::Biome::Forest) => "Forest",
            Ok(cube_worldgen::v2::Biome::Desert) => "Desert",
            Ok(cube_worldgen::v2::Biome::Tundra) => "Tundra",
            Err(_) => "",
        }
    }

    /// `true` if the untouched base world has air at the voxel (the generator's answer; edits
    /// by the player are not consulted). Unknown versions count as not free.
    pub fn is_air(&self, x: i64, y: i64, z: i64) -> bool {
        cube_worldgen::voxel_at(self.record.seed, self.record.generator_version, x, y, z)
            .is_ok_and(|m| m == 0)
    }

    /// The den in the 16 x 16 column `(cx, cz)`, if there is one: the feet position of the
    /// creature that lives there. In worlds before terrain-v6 it is two voxels off the middle of
    /// a ruin's floor; from v6 on it is the chamber under the ruin (the guard of the deep cache).
    pub fn den_site(&self, cx: i64, cz: i64) -> Option<[f64; 3]> {
        let (rx, rz) = ((cx * 16).div_euclid(40), (cz * 16).div_euclid(40));
        let chamber = self.record.generator_version.raw() >= 6;
        for dz in 0..=1 {
            for dx in 0..=1 {
                let r = cube_worldgen::ruin_of_region(
                    self.record.seed,
                    self.record.generator_version,
                    rx + dx,
                    rz + dz,
                )
                .ok()
                .flatten();
                let Some(r) = r else { continue };
                // Offsets from the ruin's centre column; the altar stands in the middle.
                let (ox, oz, y) = if chamber {
                    (2, 3, r.floor + cube_worldgen::v6::CHAMBER_FLOOR + 1)
                } else {
                    (2, 2, r.floor + 1)
                };
                if (r.x + ox).div_euclid(16) == cx && (r.z + oz).div_euclid(16) == cz {
                    return Some([
                        r.x as f64 + ox as f64 + 0.5,
                        y as f64,
                        r.z as f64 + oz as f64 + 0.5,
                    ]);
                }
            }
        }
        None
    }

    /// A cache site within `reach` voxels horizontally and a few vertically of `feet`: the altar
    /// in the middle of a ruin's floor, or (generator version 6 and later) the pedestal of the
    /// chamber under it. Returns the column that identifies the cache and whether it is the deep
    /// one. `None` where there is no ruin (also in worlds older than terrain-v5).
    pub fn altar_near(&self, feet: [f64; 3], reach: f64) -> Option<((i64, i64), bool)> {
        let rx = (feet[0].floor() as i64).div_euclid(40);
        let rz = (feet[2].floor() as i64).div_euclid(40);
        let deep_exists = self.record.generator_version.raw() >= 6;
        for dz in -1..=1 {
            for dx in -1..=1 {
                let r = cube_worldgen::ruin_of_region(
                    self.record.seed,
                    self.record.generator_version,
                    rx + dx,
                    rz + dz,
                )
                .ok()
                .flatten();
                let Some(r) = r else { continue };
                let near = |x: i64, z: i64, y: f64, dy: f64| {
                    (feet[0] - (x as f64 + 0.5)).hypot(feet[2] - (z as f64 + 0.5)) <= reach
                        && (feet[1] - y).abs() <= dy
                };
                if near(r.x, r.z, (r.floor + 1) as f64, 3.0) {
                    return Some(((r.x, r.z), false));
                }
                if deep_exists {
                    let (px, pz) = cube_worldgen::v6::PEDESTAL;
                    let y = (r.floor + cube_worldgen::v6::CHAMBER_FLOOR + 1) as f64;
                    if near(r.x + px, r.z + pz, y, 2.0) {
                        return Some(((r.x + px, r.z + pz), true));
                    }
                }
            }
        }
        None
    }

    /// The record of this world.
    pub fn record(&self) -> &WorldRecord {
        &self.record
    }

    /// The loaded static area.
    pub fn area(&self) -> StaticArea {
        self.area
    }

    /// Read access to the world.
    pub fn reader(&self) -> SharedAsset {
        self.reader.clone()
    }

    /// The Processing executor with the faces provider.
    pub fn executor(&self) -> Arc<Executor> {
        self.executor.clone()
    }

    /// Registers a listener that hears about every commit's dirty regions.
    pub fn add_dirty_listener(&self, listener: Arc<dyn DirtySink>) {
        lock(&self.fanout.listeners).push(listener);
    }

    /// Content hash of the world (for tests and diagnostics).
    pub fn content_hash(&self) -> u128 {
        self.reader.snapshot().export_view().content_hash().value()
    }

    /// Applies an edit request through the Editing API and Orchestration (the single user
    /// commit path). `Ok` means the change is committed and journalled.
    pub fn submit_edit(&self, edit: EditRequest) -> Result<(), HostError> {
        let seq = self.reserve_edit_seq();
        self.submit_edit_reserved(edit, seq)
    }

    /// Like [`LocalHost::submit_edit`] under a sequence taken earlier with
    /// [`LocalHost::reserve_edit_seq`], so that the caller can record the operation id before the
    /// change is made. Submitting the same sequence again is an idempotent retry: the world is
    /// changed at most once.
    pub fn submit_edit_reserved(&self, edit: EditRequest, seq: u64) -> Result<(), HostError> {
        let (voxel, kind, parameters) = match edit {
            EditRequest::Place { voxel, material } if material != 0 => {
                (voxel, kinds::ADD_GEOMETRY, add(material))
            }
            EditRequest::Place { .. } => return Err(HostError::Edit("material 0 is air".into())),
            EditRequest::Remove { voxel } => (voxel, kinds::REMOVE_GEOMETRY, remove(None)),
        };
        let edit_err = |e: &dyn std::fmt::Debug| HostError::Edit(format!("{e:?}"));
        let brick = BrickCoord::new(
            voxel[0].div_euclid(16),
            voxel[1].div_euclid(16),
            voxel[2].div_euclid(16),
        );
        let end = |c: i64| {
            c.checked_add(1)
                .ok_or_else(|| HostError::Edit("coordinate at the end of the grid".into()))
        };
        let region = Region::new(
            0,
            VoxelPos::new(voxel[0], voxel[1], voxel[2]),
            VoxelPos::new(end(voxel[0])?, end(voxel[1])?, end(voxel[2])?),
        )
        .map_err(|e| edit_err(&e))?;
        let op = EditingOperation {
            id: OperationId::new(PLAYER_SCOPE, seq),
            kind,
            asset: AssetId::new(ASSET),
            layers: vec![LayerId::new(VOXEL_LAYER)],
            parameters,
            read_region: None,
            write_region: region,
            conversion: None,
            mode: EditMode::Destructive,
        };
        let ctx = AuthorizationContext {
            actor: ActorId(1),
            session: SessionId(1),
            origin: Origin::LocalClient,
            capabilities: vec![EDIT],
            scope: Scope {
                assets: vec![AssetId::new(ASSET)],
                regions: vec![],
            },
        };
        let h = self
            .svc
            .begin(ctx, TransactionSpec::new(AssetId::new(ASSET)))
            .map_err(|e| edit_err(&e))?;
        let r = self.svc.execute(&h, op);
        if !r.is_success() {
            let _ = self.svc.rollback(&h);
            return Err(edit_err(&r));
        }
        match self.svc.commit(&h) {
            EditingResult::Success(_) => {
                lock(&self.streamed).edited.insert(brick); // pinned from now on
                Ok(())
            }
            other => Err(edit_err(&other)),
        }
    }
}

fn materialise(
    system: &SystemCommit,
    record: &WorldRecord,
    bricks: &[BrickCoord],
    streamed: &mut Streamed,
) -> Result<(), HostError> {
    for &b in bricks {
        if !streamed.loaded.insert(b) {
            continue;
        }
        let g = generate_brick(record.seed, record.generator_version, b)
            .map_err(HostError::Generator)?;
        let op = OperationId::new(0, streamed.next_op);
        streamed.next_op += 1;
        streamed.loads += 1;
        system
            .materialise(op, LayerId::new(VOXEL_LAYER), 0, b, g.into_change())
            .map_err(setup)?;
    }
    Ok(())
}

/// Raises the revision of a resident, unedited base brick to `want` by unloading and loading it
/// again (each load is one revision), exactly the steps the earlier session took. Fails if the
/// brick is already past `want` (the journal and the world disagree).
fn bring_to_revision(
    system: &SystemCommit,
    reader: &SharedAsset,
    record: &WorldRecord,
    b: BrickCoord,
    want: cube_core::Revision,
    streamed: &mut Streamed,
) -> Result<(), HostError> {
    let layer = LayerId::new(VOXEL_LAYER);
    let have = |reader: &SharedAsset| {
        reader
            .snapshot()
            .cell_token(layer, 0, b)
            .map_or(0, |t| t.value().raw())
    };
    while have(reader) < want.raw() {
        system.evict(layer, 0, b).map_err(setup)?;
        streamed.loaded.remove(&b);
        materialise(system, record, &[b], streamed)?;
    }
    if have(reader) != want.raw() {
        return Err(HostError::Setup(format!(
            "brick {b:?} is at revision {} but the journal expects {}",
            have(reader),
            want.raw()
        )));
    }
    Ok(())
}
