//! Shared fixtures of the Processing suites: a live asset with commit handles, and a few
//! providers (including ones that can be paused, can fail and can panic).
//!
//! This file belongs to the integration-test crate, the only place that may name the commit
//! handles (ADG, mutation boundary); `cube-processing` itself never can.
#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use cube_core::{
    AssetHost, AssetId, BrickBounds, BrickCoord, CellChange, CellKey, CellPayload, CellStore,
    CellWrite, ChannelSet, CommitBatch, CommitOutcome, ExpectedRevision, GeometryAsset,
    GeometryTransform, Layer, LayerData, LayerId, LayerKind, MaterialIndex, OperationId,
    SharedAsset, UserCommit, WriteSet,
};
use cube_processing::{
    CancellationToken, ChangeMask, ComputeContext, DerivedKind, DerivedTarget, Payload, Provider,
    ProviderFailure,
};

pub const L_VOXEL: u64 = 1;
pub const L_BOUNDED: u64 = 2;
pub const L_MESH: u64 = 3;

pub struct World {
    pub user: UserCommit,
    pub reader: SharedAsset,
    op: AtomicU64,
}

pub fn bc(x: i64, y: i64, z: i64) -> BrickCoord {
    BrickCoord::new(x, y, z)
}

pub fn key(layer: u64, x: i64, y: i64, z: i64) -> CellKey {
    CellKey::new(LayerId::new(layer), 0, bc(x, y, z)).unwrap()
}

pub fn target(kind: u32, layer: u64, x: i64, y: i64, z: i64) -> DerivedTarget {
    DerivedTarget {
        kind: DerivedKind(kind),
        key: key(layer, x, y, z),
    }
}

pub fn world() -> World {
    let bounds = BrickBounds::new(bc(-2, -2, -2), bc(2, 2, 2)).unwrap();
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
    let host = AssetHost::new(GeometryAsset::new(AssetId::new(1), layers).unwrap());
    let reader = host.reader();
    let handles = host.into_handles();
    World {
        user: handles.user,
        reader,
        op: AtomicU64::new(1),
    }
}

fn material(m: u16) -> CellPayload {
    CellPayload::Channels(ChannelSet::uniform_material(MaterialIndex::from_raw(m)))
}

impl World {
    fn next_op(&self) -> OperationId {
        OperationId::new(1, self.op.fetch_add(1, Ordering::SeqCst))
    }

    fn write(&self, layer: u64, b: BrickCoord, change: CellChange) -> CommitOutcome {
        let state = self.reader.snapshot();
        let expected = state
            .cell_token(LayerId::new(layer), 0, b)
            .map_or(ExpectedRevision::Absent, |t| {
                ExpectedRevision::At(t.value())
            });
        let batch = CommitBatch {
            sets: vec![WriteSet {
                layer: LayerId::new(layer),
                writes: vec![CellWrite {
                    level: 0,
                    brick: b,
                    expected,
                    change,
                }],
            }],
        };
        self.user.commit(self.next_op(), &batch).unwrap()
    }

    /// Writes a uniform-material brick (retries nothing: callers use one writer per brick).
    pub fn put(&self, layer: u64, b: BrickCoord, m: u16) -> CommitOutcome {
        self.write(layer, b, CellChange::Put(material(m)))
    }

    /// Writes two bricks of the voxel layer with the same material in ONE atomic commit.
    pub fn put_pair(&self, a: BrickCoord, b: BrickCoord, m: u16) -> CommitOutcome {
        let state = self.reader.snapshot();
        let layer = LayerId::new(L_VOXEL);
        let w = |brick: BrickCoord| CellWrite {
            level: 0,
            brick,
            expected: state
                .cell_token(layer, 0, brick)
                .map_or(ExpectedRevision::Absent, |t| {
                    ExpectedRevision::At(t.value())
                }),
            change: CellChange::Put(material(m)),
        };
        let batch = CommitBatch {
            sets: vec![WriteSet {
                layer,
                writes: vec![w(a), w(b)],
            }],
        };
        self.user.commit(self.next_op(), &batch).unwrap()
    }

    /// Marks a brick as loaded and empty.
    pub fn empty(&self, layer: u64, b: BrickCoord) -> CommitOutcome {
        self.write(layer, b, CellChange::KnownEmpty)
    }

    /// Revision of a brick of the voxel layer (`None` = never committed).
    pub fn rev(&self, b: BrickCoord) -> Option<u64> {
        self.reader
            .snapshot()
            .cell_token(LayerId::new(L_VOXEL), 0, b)
            .map(|t| t.value().raw())
    }
}

/// A one-shot gate: `wait` blocks until `open` is called.
#[derive(Default)]
pub struct Gate {
    open: Mutex<bool>,
    cv: Condvar,
}

impl Gate {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    pub fn open(&self) {
        *self.open.lock().unwrap() = true;
        self.cv.notify_all();
    }
    pub fn wait(&self) {
        let mut g = self.open.lock().unwrap();
        while !*g {
            g = self.cv.wait(g).unwrap();
        }
    }
    pub fn is_open(&self) -> bool {
        *self.open.lock().unwrap()
    }
}

/// Sum provider: the payload is `Tagged { revision, sum }` where `revision` is the committed
/// revision of the centre brick as the snapshot saw it and `sum` is the sum of the first
/// material of every `Present` brick in the neighbourhood (apron 1).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Tagged {
    pub revision: Option<u64>,
    pub sum: u64,
    pub present: u64,
}

pub struct SumProvider {
    pub kind: u32,
    pub entered: Option<Arc<Gate>>,
    pub release: Option<Arc<Gate>>,
    pub honour_cancel: bool,
}

impl SumProvider {
    pub fn plain(kind: u32) -> Arc<dyn Provider> {
        Arc::new(Self {
            kind,
            entered: None,
            release: None,
            honour_cancel: true,
        })
    }
    pub fn paused(
        kind: u32,
        entered: &Arc<Gate>,
        release: &Arc<Gate>,
        honour_cancel: bool,
    ) -> Arc<dyn Provider> {
        Arc::new(Self {
            kind,
            entered: Some(Arc::clone(entered)),
            release: Some(Arc::clone(release)),
            honour_cancel,
        })
    }
}

impl Provider for SumProvider {
    fn kind(&self) -> DerivedKind {
        DerivedKind(self.kind)
    }
    fn name(&self) -> &str {
        "sum"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        ChangeMask::ALL
    }
    fn apron(&self) -> u32 {
        1
    }
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        if let Some(g) = &self.entered {
            g.open();
        }
        if let Some(g) = &self.release {
            g.wait();
        }
        if self.honour_cancel {
            ctx.token.check()?;
        }
        let centre = ctx.target.key.brick();
        let mut sum = 0u64;
        let mut present = 0u64;
        for (b, state) in ctx.snapshot.cells() {
            if let Some(CellPayload::Channels(set)) = state.payload() {
                present += 1;
                let m = set
                    .material(cube_core::LocalPos::new(0, 0, 0).unwrap())
                    .map_or(0, |m| u64::from(m.raw()));
                sum += m;
            }
            let _ = b;
        }
        let revision = ctx
            .snapshot
            .dependency()
            .inputs()
            .iter()
            .find(|(b, _)| *b == centre)
            .and_then(|(_, r)| r.map(|r| r.raw()));
        Ok(Arc::new(Tagged {
            revision,
            sum,
            present,
        }))
    }
}

/// Doubles the sum of the input kind `input` (declared dependency).
pub struct DoubleOf {
    pub kind: u32,
    pub deps: Vec<DerivedKind>,
}

impl DoubleOf {
    pub fn of(kind: u32, input: u32) -> Arc<dyn Provider> {
        Arc::new(Self {
            kind,
            deps: vec![DerivedKind(input)],
        })
    }
}

impl Provider for DoubleOf {
    fn kind(&self) -> DerivedKind {
        DerivedKind(self.kind)
    }
    fn name(&self) -> &str {
        "double-of"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        ChangeMask::NONE
    }
    fn depends_on(&self) -> &[DerivedKind] {
        &self.deps
    }
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        let input = ctx
            .input(self.deps[0])
            .ok_or_else(|| ProviderFailure::Failed("input missing".into()))?;
        let t = input
            .downcast_ref::<Tagged>()
            .ok_or_else(|| ProviderFailure::Failed("wrong input type".into()))?;
        Ok(Arc::new(Tagged {
            revision: t.revision,
            sum: t.sum * 2,
            present: t.present,
        }))
    }
}

/// A provider that fails, panics, or spins until cancelled, depending on `mode`.
#[derive(Clone, Copy)]
pub enum Mode {
    Fail,
    Panic,
    SpinUntilCancelled,
}

pub struct Odd {
    pub kind: u32,
    pub mode: Mode,
    pub started: Arc<Gate>,
}

impl Provider for Odd {
    fn kind(&self) -> DerivedKind {
        DerivedKind(self.kind)
    }
    fn name(&self) -> &str {
        "odd"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        ChangeMask::ALL
    }
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        self.started.open();
        match self.mode {
            Mode::Fail => Err(ProviderFailure::Failed("boom".into())),
            Mode::Panic => panic!("provider bug"),
            Mode::SpinUntilCancelled => {
                let token: &CancellationToken = ctx.token;
                while !token.is_cancelled() {
                    std::thread::yield_now();
                }
                Err(ProviderFailure::Cancelled(token.reason().unwrap()))
            }
        }
    }
}
