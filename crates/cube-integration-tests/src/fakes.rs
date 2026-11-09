//! Test doubles and the full-stack fixture for the Orchestration suites.
//!
//! The fixture is the *real* path: a real Core asset, the real `OrchestrationService`, the real
//! single user commit path. Only the things outside PS-4 are fakes: a reference backend for the
//! voxel representation (real backends are PS-5), a journal with failure injection (the durable
//! journal is PS-10) and recording observers.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use cube_core::{
    AssetHost, AssetId, BrickBounds, BrickCoord, CellChange, CellKey, CellPayload, CellStore,
    CellWrite, ChannelSet, DirtyRegion, ExpectedRevision, GeometryAsset, GeometryTransform, Layer,
    LayerData, LayerId, LayerKind, MaterialIndex, OperationId, RecoveryCommit, Region, SharedAsset,
    SystemCommit, VoxelPos, WriteSet,
};
use cube_editing_api::{
    ActorId, AuthorizationContext, BackendCapabilities, Capability, DeltaId, EditMode,
    EditingError, EditingOperation, EditingResult, EditingService, GeometryBackend, GeometryDelta,
    JournalEntry, JournalPhase, JournalSeq, KindId, OperationJournal, Origin, OverlayRead,
    Parameters, Scope, TransactionObserver, TransactionSpec, TransactionSummary, ValidationReport,
    kinds,
};
use cube_orchestration::{
    BackendRegistry, CapabilityAuthorizer, CommitPath, Config, DirtySink, OrchestrationService,
};
use cube_processing::Snapshot;

/// The capability that allows editing in the fixture's policy.
pub const EDIT: Capability = Capability(1);
/// Voxel layer without bounds.
pub const L_VOXEL: u64 = 1;
/// Voxel layer with bounds `[-2, 2)` bricks.
pub const L_BOUNDED: u64 = 2;
/// A mesh layer (no backend is registered for it).
pub const L_MESH: u64 = 3;
/// The asset id.
pub const ASSET: u64 = 1;

/// Shorthand for a brick coordinate.
pub fn bc(x: i64, y: i64, z: i64) -> BrickCoord {
    BrickCoord::new(x, y, z)
}

/// A context holding the edit capability for the fixture's asset.
pub fn ctx(origin: Origin) -> AuthorizationContext {
    AuthorizationContext {
        actor: ActorId(42),
        session: cube_editing_api::SessionId(1),
        origin,
        capabilities: vec![EDIT],
        scope: Scope {
            assets: vec![AssetId::new(ASSET)],
            regions: vec![],
        },
    }
}

/// A transaction spec for the fixture's asset.
pub fn spec() -> TransactionSpec {
    TransactionSpec::new(AssetId::new(ASSET))
}

/// The voxel-space box covering the bricks `lo..=hi` at level 0.
pub fn bricks_region(lo: BrickCoord, hi: BrickCoord) -> Region {
    Region::new(
        0,
        VoxelPos::new(lo.x * 16, lo.y * 16, lo.z * 16),
        VoxelPos::new((hi.x + 1) * 16, (hi.y + 1) * 16, (hi.z + 1) * 16),
    )
    .expect("a valid box")
}

/// An operation of `kind` on `layer` over the bricks `lo..=hi` with `material` as parameter.
pub fn op(
    n: u64,
    kind: KindId,
    layer: u64,
    lo: BrickCoord,
    hi: BrickCoord,
    material: u16,
) -> EditingOperation {
    EditingOperation {
        id: OperationId::new(7, n),
        kind,
        asset: AssetId::new(ASSET),
        layers: vec![LayerId::new(layer)],
        parameters: Parameters {
            schema: None,
            bytes: material.to_le_bytes().to_vec(),
        },
        read_region: None,
        write_region: bricks_region(lo, hi),
        conversion: None,
        mode: EditMode::Destructive,
    }
}

/// A one-brick `SET_MATERIAL` on the unbounded voxel layer.
pub fn set(n: u64, b: BrickCoord, material: u16) -> EditingOperation {
    op(n, kinds::SET_MATERIAL, L_VOXEL, b, b, material)
}

fn material_of(p: &Parameters) -> u16 {
    match p.bytes.as_slice() {
        [a, b, ..] => u16::from_le_bytes([*a, *b]),
        _ => 1,
    }
}

/// Reference backend of the voxel representation: `SET_MATERIAL` and `ADD_GEOMETRY` fill every
/// brick of the write region with one material, `REMOVE_GEOMETRY` marks them empty.
pub struct SimpleBackend {
    caps: BackendCapabilities,
}

impl SimpleBackend {
    /// The backend as a shareable port.
    pub fn arc() -> Arc<dyn GeometryBackend> {
        Arc::new(Self {
            caps: BackendCapabilities {
                layer_kind: LayerKind::Voxel,
                kinds: [
                    kinds::SET_MATERIAL,
                    kinds::ADD_GEOMETRY,
                    kinds::REMOVE_GEOMETRY,
                ]
                .into_iter()
                .collect(),
                native_boolean: false,
                resample: false,
                validate: true,
            },
        })
    }
}

impl GeometryBackend for SimpleBackend {
    fn capabilities(&self) -> &BackendCapabilities {
        &self.caps
    }

    fn apply(
        &self,
        o: &EditingOperation,
        _: &dyn OverlayRead,
        _: Option<&Snapshot>,
    ) -> Result<Vec<WriteSet>, EditingError> {
        if !self.caps.supports(o.kind) {
            return Err(EditingError::UnsupportedGeometryOperation(o.kind));
        }
        let change = if o.kind == kinds::REMOVE_GEOMETRY {
            CellChange::KnownEmpty
        } else {
            CellChange::Put(CellPayload::Channels(ChannelSet::uniform_material(
                MaterialIndex::from_raw(material_of(&o.parameters)),
            )))
        };
        Ok(o.layers
            .iter()
            .map(|layer| WriteSet {
                layer: *layer,
                writes: o
                    .write_region
                    .bricks()
                    .map(|brick| CellWrite {
                        level: o.write_region.level(),
                        brick,
                        expected: ExpectedRevision::Absent,
                        change: change.clone(),
                    })
                    .collect(),
            })
            .collect())
    }

    fn validate(&self, _: &dyn OverlayRead, _: &Region) -> Result<ValidationReport, EditingError> {
        Ok(ValidationReport::default())
    }
}

/// A backend whose behaviour is a closure, for negative tests of the write-set checks.
pub struct ScriptedBackend {
    caps: BackendCapabilities,
    #[allow(clippy::type_complexity)]
    script: Box<dyn Fn(&EditingOperation) -> Result<Vec<WriteSet>, EditingError> + Send + Sync>,
}

impl ScriptedBackend {
    /// A voxel backend advertising `SET_MATERIAL` that runs `script`.
    #[allow(clippy::type_complexity)]
    pub fn arc(
        script: impl Fn(&EditingOperation) -> Result<Vec<WriteSet>, EditingError>
        + Send
        + Sync
        + 'static,
    ) -> Arc<dyn GeometryBackend> {
        Arc::new(Self {
            caps: BackendCapabilities {
                layer_kind: LayerKind::Voxel,
                kinds: BTreeSet::from([kinds::SET_MATERIAL]),
                native_boolean: false,
                resample: false,
                validate: false,
            },
            script: Box::new(script),
        })
    }
}

impl GeometryBackend for ScriptedBackend {
    fn capabilities(&self) -> &BackendCapabilities {
        &self.caps
    }
    fn apply(
        &self,
        o: &EditingOperation,
        _: &dyn OverlayRead,
        _: Option<&Snapshot>,
    ) -> Result<Vec<WriteSet>, EditingError> {
        (self.script)(o)
    }
    fn validate(&self, _: &dyn OverlayRead, _: &Region) -> Result<ValidationReport, EditingError> {
        Ok(ValidationReport::default())
    }
}

/// A uniform-material write of one cell, for scripted backends.
pub fn cell_write(layer: u64, b: BrickCoord, material: u16) -> WriteSet {
    WriteSet {
        layer: LayerId::new(layer),
        writes: vec![CellWrite {
            level: 0,
            brick: b,
            expected: ExpectedRevision::Absent,
            change: CellChange::Put(CellPayload::Channels(ChannelSet::uniform_material(
                MaterialIndex::from_raw(material),
            ))),
        }],
    }
}

/// A journal in memory with failure injection per phase.
#[derive(Default)]
pub struct FaultyJournal {
    entries: Mutex<Vec<JournalEntry>>,
    /// Fail every `Intent` append.
    pub fail_intent: AtomicBool,
    /// Fail every `Confirmed` append.
    pub fail_confirm: AtomicBool,
    /// Fail every `Aborted` append.
    pub fail_abort: AtomicBool,
}

impl FaultyJournal {
    /// All records so far.
    pub fn entries(&self) -> Vec<JournalEntry> {
        self.entries
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
    /// The phases so far, in order.
    pub fn phases(&self) -> Vec<JournalPhase> {
        self.entries().iter().map(|e| e.phase).collect()
    }
}

impl OperationJournal for FaultyJournal {
    fn append(&self, mut e: JournalEntry) -> Result<JournalSeq, EditingError> {
        let fail = match e.phase {
            JournalPhase::Intent => &self.fail_intent,
            JournalPhase::Confirmed => &self.fail_confirm,
            JournalPhase::Aborted => &self.fail_abort,
        };
        if fail.load(Ordering::SeqCst) {
            return Err(EditingError::PersistenceFailure);
        }
        let mut g = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        e.seq = JournalSeq(g.len() as u64 + 1);
        let s = e.seq;
        g.push(e);
        Ok(s)
    }
    fn read_from(&self, from: JournalSeq, limit: usize) -> Result<Vec<JournalEntry>, EditingError> {
        Ok(self
            .entries()
            .into_iter()
            .filter(|e| e.seq >= from)
            .take(limit)
            .collect())
    }
}

/// Shared event log, so that sinks, journals and observers can be put in one order.
pub type EventLog = Arc<Mutex<Vec<String>>>;

/// Records every notification (and the asset revision it saw) and can be told to panic.
pub struct RecordingObserver {
    name: &'static str,
    log: EventLog,
    reader: SharedAsset,
    /// Panic on every commit notification.
    pub panic_on_commit: AtomicBool,
    /// Operation ids in delivery order.
    pub committed: Mutex<Vec<OperationId>>,
    /// Asset revision the observer saw when it was told, per commit.
    pub seen_revisions: Mutex<Vec<u64>>,
}

impl RecordingObserver {
    /// A new observer writing into `log`.
    pub fn new(name: &'static str, log: &EventLog, reader: &SharedAsset) -> Arc<Self> {
        Arc::new(Self {
            name,
            log: Arc::clone(log),
            reader: reader.clone(),
            panic_on_commit: AtomicBool::new(false),
            committed: Mutex::new(Vec::new()),
            seen_revisions: Mutex::new(Vec::new()),
        })
    }
}

impl TransactionObserver for RecordingObserver {
    fn on_committed(&self, s: &TransactionSummary, d: &GeometryDelta) {
        if self.panic_on_commit.load(Ordering::SeqCst) {
            panic!("observer bug");
        }
        let rev = self.reader.snapshot().asset_token().value().raw();
        self.seen_revisions
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(rev);
        self.log
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(format!(
                "{}:commit:{}:{}",
                self.name, s.transaction.0, d.id.0
            ));
        self.committed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(d.operation);
    }
    fn on_rolled_back(&self, s: &TransactionSummary) {
        self.log
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(format!("{}:rollback:{}", self.name, s.transaction.0));
    }
}

/// A dirty sink that records what it was given.
pub struct RecordingSink {
    log: EventLog,
    /// Every batch of dirty regions, in order.
    pub batches: Mutex<Vec<Vec<DirtyRegion>>>,
    forward: Mutex<Option<Arc<dyn DirtySink>>>,
}

impl RecordingSink {
    /// A sink writing `sink` entries into `log`.
    pub fn new(log: &EventLog) -> Arc<Self> {
        Arc::new(Self {
            log: Arc::clone(log),
            batches: Mutex::new(Vec::new()),
            forward: Mutex::new(None),
        })
    }

    /// Also passes every batch on to `next` (for example a Processing executor).
    pub fn forward_to(&self, next: Arc<dyn DirtySink>) {
        *self.forward.lock().unwrap_or_else(PoisonError::into_inner) = Some(next);
    }
}

impl DirtySink for RecordingSink {
    fn dirty(&self, regions: &[DirtyRegion]) {
        self.log
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(format!("sink:{}", regions.len()));
        self.batches
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(regions.to_vec());
        let next = self
            .forward
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(n) = next {
            n.dirty(regions);
        }
    }
}

/// Options of the fixture.
pub struct Options {
    /// Backends to register (default: the reference voxel backend).
    pub backends: Vec<Arc<dyn GeometryBackend>>,
    /// Use a journal.
    pub journal: bool,
    /// How many committed operation ids are remembered for retry.
    pub retry_capacity: usize,
    /// Start from a restored asset view instead of the default world.
    pub restore: Option<cube_core::AssetView>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            backends: vec![SimpleBackend::arc()],
            journal: false,
            retry_capacity: 1024,
            restore: None,
        }
    }
}

/// The real path, assembled.
pub struct Fixture {
    /// The editing service under test.
    pub svc: Arc<OrchestrationService>,
    /// Read access to the world.
    pub reader: SharedAsset,
    /// The WorldGen writer (a different category from the user path).
    pub system: SystemCommit,
    /// The recovery writer.
    pub recovery: RecoveryCommit,
    /// The journal (even when not wired in, so tests can inspect it).
    pub journal: Arc<FaultyJournal>,
    /// First observer.
    pub obs1: Arc<RecordingObserver>,
    /// Second observer.
    pub obs2: Arc<RecordingObserver>,
    /// Dirty sink.
    pub sink: Arc<RecordingSink>,
    /// The shared event log.
    pub log: EventLog,
}

/// Builds the real path with default options.
pub fn fixture() -> Fixture {
    fixture_with(Options::default())
}

/// Builds the real path.
pub fn fixture_with(o: Options) -> Fixture {
    let bounds = BrickBounds::new(bc(-2, -2, -2), bc(2, 2, 2)).expect("bounds");
    let layers = vec![
        Layer::new(
            LayerId::new(L_VOXEL),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Voxel),
        ),
        Layer::new(
            LayerId::new(L_BOUNDED),
            GeometryTransform::IDENTITY,
            LayerData::Voxel(CellStore::with_bounds(bounds)),
        ),
        Layer::new(
            LayerId::new(L_MESH),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Mesh),
        ),
    ];
    let host = match &o.restore {
        Some(v) => AssetHost::from_view(v).expect("a valid view"),
        None => AssetHost::new(GeometryAsset::new(AssetId::new(ASSET), layers).expect("asset")),
    };
    let reader = host.reader();
    let handles = host.into_handles();
    let log: EventLog = Arc::default();
    let journal = Arc::new(FaultyJournal::default());
    let obs1 = RecordingObserver::new("o1", &log, &reader);
    let obs2 = RecordingObserver::new("o2", &log, &reader);
    let sink = RecordingSink::new(&log);
    let mut registry = BackendRegistry::new();
    for b in o.backends {
        registry.register(b).expect("backend");
    }
    let svc = Arc::new(OrchestrationService::new(Config {
        path: CommitPath::new(handles.user),
        registry,
        authorizer: Arc::new(CapabilityAuthorizer { edit: EDIT }),
        journal: if o.journal {
            Some(journal.clone() as Arc<dyn OperationJournal>)
        } else {
            None
        },
        dirty: Some(sink.clone() as Arc<dyn DirtySink>),
        observers: vec![
            obs1.clone() as Arc<dyn TransactionObserver>,
            obs2.clone() as Arc<dyn TransactionObserver>,
        ],
        retry_capacity: o.retry_capacity,
    }));
    Fixture {
        svc,
        reader,
        system: handles.system,
        recovery: handles.recovery,
        journal,
        obs1,
        obs2,
        sink,
        log,
    }
}

impl Fixture {
    /// Runs `ops` in one transaction and commits it.
    pub fn run(&self, ops: Vec<EditingOperation>) -> EditingResult {
        let h = self.svc.begin(ctx(Origin::Server), spec()).expect("begin");
        for o in ops {
            let r = self.svc.execute(&h, o);
            if !r.is_success() {
                let _ = self.svc.rollback(&h);
                return r;
            }
        }
        self.svc.commit(&h)
    }

    /// Material of the first channel at local (0,0,0) of a brick of the unbounded voxel layer;
    /// `None` if the brick is not present.
    pub fn material(&self, b: BrickCoord) -> Option<u16> {
        self.material_in(L_VOXEL, b)
    }

    /// Same for any layer.
    pub fn material_in(&self, layer: u64, b: BrickCoord) -> Option<u16> {
        let state = self.reader.snapshot();
        match state.cell(LayerId::new(layer), 0, b)?.ok()? {
            cube_core::CellState::Present(p) => match &*p {
                CellPayload::Channels(set) => set
                    .material(cube_core::LocalPos::new(0, 0, 0).ok()?)
                    .map(|m| m.raw()),
                _ => None,
            },
            _ => None,
        }
    }

    /// Asset revision.
    pub fn revision(&self) -> u64 {
        self.reader.snapshot().asset_token().value().raw()
    }

    /// Number of changing commits so far.
    pub fn commits(&self) -> u64 {
        self.reader.commit_log().0
    }

    /// The key of a brick of the unbounded voxel layer.
    pub fn key(&self, b: BrickCoord) -> CellKey {
        CellKey::new(LayerId::new(L_VOXEL), 0, b).expect("key")
    }
}

/// A delta id for tests that need one.
pub const fn delta(n: u64) -> DeltaId {
    DeltaId(n)
}
