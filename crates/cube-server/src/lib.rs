//! The in-process local host of one world (first world; DEC-092). Not the networked server
//! (PS-13), not a child process: a library that the application process calls.
//!
//! It owns everything authoritative about a local world and nothing about how it is shown:
//! the Core asset and its handles, Orchestration with the voxel backend, the world record, the
//! manifest and the file journal, the generator and the Processing executor (faces provider).
//!
//! * the **untouched base** is never saved: it is regenerated from the record's seed and
//!   generator version into a **static bounded area** (not streaming; WGEN-02..04 are not
//!   built), in a fixed order, by the system-commit port, which never overwrites a loaded cell;
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
use cube_core::{
    AssetHost, AssetId, BrickCoord, DirtyRegion, GeometryAsset, GeometryTransform, Layer,
    LayerData, LayerId, LayerKind, OperationId, Region, SharedAsset, SystemCommit, VoxelPos,
};
use cube_editing_api::{
    ActorId, AuthorizationContext, Capability, EditMode, EditingOperation, EditingResult,
    EditingService, JournalSeq, OperationJournal, Origin, Scope, SessionId, TransactionSpec, kinds,
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

/// The one asset of a local world.
pub const ASSET: u64 = 1;
/// The voxel layer.
pub const VOXEL_LAYER: u64 = 1;
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
        let h = cube_worldgen::v1::height(record.seed, 0, 0);
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
        let mut st = DirStorage::open(dir).map_err(persistence)?;
        create_world_with_record(&mut st, &record, &Limits::default()).map_err(persistence)?;
        Self::open(dir, area)
    }

    /// Opens the world in `dir`: reads the record, regenerates the base area, replays the
    /// confirmed edits and rebuilds the retry table. The generator version of the record is
    /// never substituted.
    pub fn open(dir: &Path, area: StaticArea) -> Result<Self, HostError> {
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
        if snapshot.is_none() {
            materialise(&handles.system, &record, &area)?;
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
        Ok(Self {
            record,
            area,
            reader,
            svc,
            executor,
            fanout,
            next_seq: Mutex::new(next_seq),
        })
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
        let (voxel, kind, parameters) = match edit {
            EditRequest::Place { voxel, material } if material != 0 => {
                (voxel, kinds::ADD_GEOMETRY, add(material))
            }
            EditRequest::Place { .. } => return Err(HostError::Edit("material 0 is air".into())),
            EditRequest::Remove { voxel } => (voxel, kinds::REMOVE_GEOMETRY, remove(None)),
        };
        let edit_err = |e: &dyn std::fmt::Debug| HostError::Edit(format!("{e:?}"));
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
        let seq = {
            let mut n = lock(&self.next_seq);
            let s = *n;
            *n += 1;
            s
        };
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
            EditingResult::Success(_) => Ok(()),
            other => Err(edit_err(&other)),
        }
    }
}

fn materialise(
    system: &SystemCommit,
    record: &WorldRecord,
    area: &StaticArea,
) -> Result<(), HostError> {
    for (n, b) in area.bricks().into_iter().enumerate() {
        let g = generate_brick(record.seed, record.generator_version, b)
            .map_err(HostError::Generator)?;
        system
            .materialise(
                OperationId::new(0, n as u64 + 1),
                LayerId::new(VOXEL_LAYER),
                0,
                b,
                g.into_change(),
            )
            .map_err(setup)?;
    }
    Ok(())
}
