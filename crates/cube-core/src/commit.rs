//! Commit: immutable asset state, the commit gate and `CommitBatch` (CORE-06, contracts C-01, C-18;
//! decision OD-R2 / DEC-061).
//!
//! Readers take cheap immutable snapshots. Writers are serialised by a gate: a batch is validated
//! completely (nothing is mutated), then applied in one step, or rejected with every conflict.
//! Revisions grow by exactly +1 per touched scope per commit; a batch that changes nothing changes
//! no token and produces no dirty region.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, PoisonError};

#[cfg(loom)]
use loom::sync::{Mutex, RwLock};
#[cfg(not(loom))]
use std::sync::{Mutex, RwLock};

use crate::asset::GeometryAsset;
use crate::cell::{CellPayload, CellState, CellStore, ChannelId, ChannelSet};
use crate::coord::{BrickCoord, VoxelPos, check_level};
use crate::dirty::{ChangeType, DirtyRegion};
use crate::error::CoreError;
use crate::ids::{AssetId, LayerId};
use crate::layer::{Layer, LayerData, LayerKind};
use crate::operation::{ActorKind, OperationId};
use crate::region::Region;
use crate::revision::{Epoch, ResourceId, Revision, RevisionToken};

/// What the writer expects the cell's committed revision to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedRevision {
    /// The cell has never been committed.
    Absent,
    /// The cell's current revision is exactly this.
    At(Revision),
}

/// New content of a cell. Commits write only these two states (never `NotLoaded` and the like).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellChange {
    /// A present payload.
    Put(CellPayload),
    /// Loaded and empty.
    KnownEmpty,
}

/// One cell write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellWrite {
    /// Refinement level of the brick.
    pub level: i8,
    /// Brick coordinate at that level.
    pub brick: BrickCoord,
    /// Expected current revision.
    pub expected: ExpectedRevision,
    /// New content.
    pub change: CellChange,
}

/// Writes to one layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteSet {
    /// Target layer.
    pub layer: LayerId,
    /// The cell writes (each cell at most once).
    pub writes: Vec<CellWrite>,
}

/// An atomic batch over the layers of one asset.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitBatch {
    /// Per-layer write sets.
    pub sets: Vec<WriteSet>,
}

/// One cell whose expectation did not hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conflict {
    /// Layer of the cell.
    pub layer: LayerId,
    /// Level of the cell.
    pub level: i8,
    /// Brick of the cell.
    pub brick: BrickCoord,
    /// What the writer expected.
    pub expected: ExpectedRevision,
    /// The actual committed revision (`None` = never committed).
    pub actual: Option<Revision>,
}

/// Why a commit was rejected. A rejected commit changes nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommitError {
    /// One or more expectations failed; all conflicting cells are listed.
    Conflict(Vec<Conflict>),
    /// The batch names a layer that does not exist.
    UnknownLayer(LayerId),
    /// The layer kind does not support cell writes yet, or the payload does not fit the kind.
    PayloadKindMismatch(LayerId),
    /// A write targets a cell outside the layer bounds.
    OutsideBounds {
        /// Layer.
        layer: LayerId,
        /// Level.
        level: i8,
        /// Brick.
        brick: BrickCoord,
    },
    /// The same cell is written twice in one batch.
    DuplicateWrite {
        /// Layer.
        layer: LayerId,
        /// Level.
        level: i8,
        /// Brick.
        brick: BrickCoord,
    },
    /// A Core-level error (invalid level, revision overflow, unrepresentable region).
    Core(CoreError),
}

impl CommitError {
    /// Stable code (range 2000..3000 belongs to commit errors; wrapped Core errors keep theirs).
    pub fn code(&self) -> crate::error::ErrorCode {
        crate::error::ErrorCode(match self {
            Self::Conflict(_) => 2001,
            Self::UnknownLayer(_) => 2002,
            Self::PayloadKindMismatch(_) => 2003,
            Self::OutsideBounds { .. } => 2004,
            Self::DuplicateWrite { .. } => 2005,
            Self::Core(e) => e.code().0,
        })
    }

    /// Severity (a wrapped Core error keeps its own).
    pub fn severity(&self) -> crate::error::Severity {
        match self {
            Self::Core(e) => e.severity(),
            _ => crate::error::Severity::Error,
        }
    }

    /// Category (a wrapped Core error keeps its own).
    pub fn category(&self) -> crate::error::ErrorCategory {
        match self {
            Self::Conflict(_) => crate::error::ErrorCategory::Conflict,
            Self::Core(e) => e.category(),
            _ => crate::error::ErrorCategory::Validation,
        }
    }
}

impl From<CoreError> for CommitError {
    fn from(e: CoreError) -> Self {
        Self::Core(e)
    }
}

/// Result of a successful commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitOutcome {
    /// `false` if the batch changed nothing (then no token moved and `dirty` is empty).
    pub changed: bool,
    /// Asset token after the commit.
    pub asset_token: RevisionToken,
    /// Tokens of the layers touched by changes.
    pub layer_tokens: Vec<RevisionToken>,
    /// Tokens of the cells that changed, in `(layer, level, brick)` order.
    pub cell_tokens: Vec<RevisionToken>,
    /// Regions that actually changed.
    pub dirty: Vec<DirtyRegion>,
}

/// Result of a materialisation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialiseOutcome {
    /// The cell was not loaded and has been written.
    Written(CommitOutcome),
    /// The cell was already loaded: nothing was changed.
    AlreadyLoaded,
}

fn missing_store_error(state: &AssetState, layer: LayerId) -> CommitError {
    if state.asset.layer(layer).is_some() {
        CommitError::PayloadKindMismatch(layer)
    } else {
        CommitError::UnknownLayer(layer)
    }
}

/// Immutable state of an asset: structure, data and all revisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetState {
    asset: GeometryAsset,
    epoch: Epoch,
    revision: Revision,
    layer_revisions: BTreeMap<LayerId, Revision>,
}

impl AssetState {
    /// A state with explicit revisions (restoring an exported view).
    pub(crate) fn from_parts(
        asset: GeometryAsset,
        epoch: Epoch,
        revision: Revision,
        layer_revisions: BTreeMap<LayerId, Revision>,
    ) -> Self {
        Self {
            asset,
            epoch,
            revision,
            layer_revisions,
        }
    }

    fn new(asset: GeometryAsset) -> Self {
        let layer_revisions = asset
            .layers()
            .iter()
            .map(|l| (l.id(), Revision::INITIAL))
            .collect();
        Self {
            asset,
            epoch: Epoch::INITIAL,
            revision: Revision::INITIAL,
            layer_revisions,
        }
    }

    /// The asset (layers and their data).
    pub fn asset(&self) -> &GeometryAsset {
        &self.asset
    }

    /// Asset-level token.
    pub fn asset_token(&self) -> RevisionToken {
        RevisionToken::new(
            ResourceId::Asset(self.asset.id()),
            self.epoch,
            self.revision,
        )
    }

    /// Token of a layer, `None` if the layer does not exist.
    pub fn layer_token(&self, layer: LayerId) -> Option<RevisionToken> {
        self.layer_revisions
            .get(&layer)
            .map(|r| RevisionToken::new(ResourceId::Layer(layer), self.epoch, *r))
    }

    /// The same state with `layer` data replaced by a new cell store (tokens unchanged).
    fn with_store(&self, layer: LayerId, store: CellStore) -> Result<Self, CoreError> {
        let layers: Vec<Layer> = self
            .asset
            .layers()
            .iter()
            .map(|l| {
                if l.id() == layer {
                    let data = match l.kind() {
                        LayerKind::Sdf => LayerData::Sdf(store.clone()),
                        _ => LayerData::Voxel(store.clone()),
                    };
                    Layer::new(l.id(), l.transform(), data)
                } else {
                    l.clone()
                }
            })
            .collect();
        Ok(Self {
            asset: GeometryAsset::new(self.asset.id(), layers)?,
            epoch: self.epoch,
            revision: self.revision,
            layer_revisions: self.layer_revisions.clone(),
        })
    }

    fn store(&self, layer: LayerId) -> Option<&CellStore> {
        match self.asset.layer(layer)?.data() {
            LayerData::Voxel(s) | LayerData::Sdf(s) => Some(s),
            LayerData::Mesh(_) => None,
        }
    }

    /// State of a cell; `None` if the layer does not exist or has no cell store.
    pub fn cell(
        &self,
        layer: LayerId,
        level: i8,
        brick: BrickCoord,
    ) -> Option<Result<CellState, CoreError>> {
        self.store(layer).map(|s| s.get(level, brick))
    }

    /// Token of a cell, `None` if it was never committed (or the layer does not exist).
    pub fn cell_token(
        &self,
        layer: LayerId,
        level: i8,
        brick: BrickCoord,
    ) -> Option<RevisionToken> {
        let rev = self.store(layer)?.revision(level, brick)?;
        let key = crate::key::CellKey::new(layer, level, brick).ok()?;
        Some(RevisionToken::new(ResourceId::Cell(key), self.epoch, rev))
    }
}

/// One entry of the commit log: every asset revision increment has exactly one entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommitLogEntry {
    /// Number of the committed (changing) commit, starting at 1.
    pub sequence: u64,
    /// Origin of the commit.
    pub actor: ActorKind,
    /// Operation of the commit.
    pub operation: OperationId,
    /// Asset token produced by the commit.
    pub asset_token: RevisionToken,
}

const LOG_CAPACITY: usize = 1024;

struct Log {
    total: u64,
    recent: std::collections::VecDeque<CommitLogEntry>,
}

struct Inner {
    id: AssetId,
    state: RwLock<Arc<AssetState>>,
    gate: Mutex<()>,
    log: Mutex<Log>,
}

/// Shared read access to an asset. It has no way to change data: commits go through the handles
/// of the mutation boundary.
#[derive(Clone)]
pub struct SharedAsset {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for SharedAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedAsset")
            .field("id", &self.inner.id)
            .finish_non_exhaustive()
    }
}

impl SharedAsset {
    pub(crate) fn from_state(state: AssetState) -> Self {
        let id = state.asset().id();
        Self {
            inner: Arc::new(Inner {
                id,
                state: RwLock::new(Arc::new(state)),
                gate: Mutex::new(()),
                log: Mutex::new(Log {
                    total: 0,
                    recent: std::collections::VecDeque::new(),
                }),
            }),
        }
    }

    pub(crate) fn new(asset: GeometryAsset) -> Self {
        let id = asset.id();
        let log = Log {
            total: 0,
            recent: std::collections::VecDeque::new(),
        };
        Self {
            inner: Arc::new(Inner {
                id,
                state: RwLock::new(Arc::new(AssetState::new(asset))),
                gate: Mutex::new(()),
                log: Mutex::new(log),
            }),
        }
    }

    /// Id of the asset.
    pub fn id(&self) -> AssetId {
        self.inner.id
    }

    /// A consistent immutable snapshot; later commits never change it.
    pub fn snapshot(&self) -> Arc<AssetState> {
        Arc::clone(
            &self
                .inner
                .state
                .read()
                .unwrap_or_else(PoisonError::into_inner),
        )
    }

    /// Total number of changing commits and the most recent log entries (up to 1024).
    pub fn commit_log(&self) -> (u64, Vec<CommitLogEntry>) {
        let l = self
            .inner
            .log
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        (l.total, l.recent.iter().copied().collect())
    }

    /// The commit primitive. Crate-private: only the boundary handles call it.
    pub(crate) fn commit(
        &self,
        actor: ActorKind,
        operation: OperationId,
        batch: &CommitBatch,
    ) -> Result<CommitOutcome, CommitError> {
        let _gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        self.commit_locked(actor, operation, batch)
    }

    /// System generation: writes `change` into a cell that is not loaded, and does nothing if the
    /// cell is already loaded (so repeating it, or racing with a user edit, never overwrites data).
    /// Atomic with respect to other commits.
    pub(crate) fn materialise(
        &self,
        operation: OperationId,
        layer: LayerId,
        level: i8,
        brick: BrickCoord,
        change: CellChange,
    ) -> Result<MaterialiseOutcome, CommitError> {
        let _gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let current = self.snapshot();
        let state = current
            .cell(layer, level, brick)
            .ok_or_else(|| missing_store_error(&current, layer))??;
        match state {
            CellState::Present(_) | CellState::KnownEmpty => {
                return Ok(MaterialiseOutcome::AlreadyLoaded);
            }
            CellState::OutsideBounds => {
                return Err(CommitError::OutsideBounds {
                    layer,
                    level,
                    brick,
                });
            }
            CellState::NotLoaded | CellState::LoadFailure(_) => {}
        }
        let expected = match current.cell_token(layer, level, brick) {
            Some(t) => ExpectedRevision::At(t.value()),
            None => ExpectedRevision::Absent,
        };
        let batch = CommitBatch {
            sets: vec![WriteSet {
                layer,
                writes: vec![CellWrite {
                    level,
                    brick,
                    expected,
                    change,
                }],
            }],
        };
        self.commit_locked(ActorKind::System, operation, &batch)
            .map(MaterialiseOutcome::Written)
    }

    /// Drops a loaded cell from memory. Tokens, the commit log and the cell's revision history are
    /// untouched (the data is not changed, only unloaded); the caller guarantees it is persisted.
    /// Returns `false` if the cell was not loaded.
    pub(crate) fn evict(
        &self,
        layer: LayerId,
        level: i8,
        brick: BrickCoord,
    ) -> Result<bool, CommitError> {
        let _gate = self
            .inner
            .gate
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let current = self.snapshot();
        let state = current
            .cell(layer, level, brick)
            .ok_or_else(|| missing_store_error(&current, layer))??;
        if !state.is_loaded() {
            return Ok(false);
        }
        let mut store = current.store(layer).cloned().unwrap_or_default();
        store.remove(level, brick);
        let next = current.with_store(layer, store)?;
        *self
            .inner
            .state
            .write()
            .unwrap_or_else(PoisonError::into_inner) = Arc::new(next);
        Ok(true)
    }

    fn commit_locked(
        &self,
        actor: ActorKind,
        operation: OperationId,
        batch: &CommitBatch,
    ) -> Result<CommitOutcome, CommitError> {
        let current = self.snapshot();
        let plan = plan_commit(&current, batch)?;
        if plan.changes.is_empty() {
            return Ok(CommitOutcome {
                changed: false,
                asset_token: current.asset_token(),
                layer_tokens: vec![],
                cell_tokens: vec![],
                dirty: vec![],
            });
        }
        let next = apply_plan(&current, plan, actor, operation)?;
        let outcome = next.1;
        let state = Arc::new(next.0);
        {
            let mut log = self
                .inner
                .log
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            log.total += 1;
            let entry = CommitLogEntry {
                sequence: log.total,
                actor,
                operation,
                asset_token: outcome.asset_token,
            };
            if log.recent.len() == LOG_CAPACITY {
                log.recent.pop_front();
            }
            log.recent.push_back(entry);
        }
        *self
            .inner
            .state
            .write()
            .unwrap_or_else(PoisonError::into_inner) = state;
        Ok(outcome)
    }
}

struct PlannedChange {
    layer: LayerId,
    level: i8,
    brick: BrickCoord,
    payload: Option<Arc<CellPayload>>,
    change: ChangeType,
}

struct Plan {
    changes: Vec<PlannedChange>,
}

/// Validates the whole batch against `state` without mutating anything.
fn plan_commit(state: &AssetState, batch: &CommitBatch) -> Result<Plan, CommitError> {
    let mut conflicts = Vec::new();
    let mut seen: BTreeSet<(LayerId, i8, BrickCoord)> = BTreeSet::new();
    let mut changes = Vec::new();
    for set in &batch.sets {
        let layer = state
            .asset
            .layer(set.layer)
            .ok_or(CommitError::UnknownLayer(set.layer))?;
        let store = match layer.data() {
            LayerData::Voxel(s) | LayerData::Sdf(s) => s,
            LayerData::Mesh(_) => return Err(CommitError::PayloadKindMismatch(set.layer)),
        };
        for w in &set.writes {
            check_level(w.level)?;
            if !seen.insert((set.layer, w.level, w.brick)) {
                return Err(CommitError::DuplicateWrite {
                    layer: set.layer,
                    level: w.level,
                    brick: w.brick,
                });
            }
            if !store.in_bounds(w.level, w.brick) {
                return Err(CommitError::OutsideBounds {
                    layer: set.layer,
                    level: w.level,
                    brick: w.brick,
                });
            }
            if let CellChange::Put(CellPayload::MeshPage(_)) = &w.change {
                return Err(CommitError::PayloadKindMismatch(set.layer));
            }
            let actual = store.revision(w.level, w.brick);
            let ok = match (w.expected, actual) {
                (ExpectedRevision::Absent, None) => true,
                (ExpectedRevision::At(e), Some(a)) => e == a,
                _ => false,
            };
            if !ok {
                conflicts.push(Conflict {
                    layer: set.layer,
                    level: w.level,
                    brick: w.brick,
                    expected: w.expected,
                    actual,
                });
                continue;
            }
            let current = store.get(w.level, w.brick)?;
            let (payload, change) = match (&w.change, &current) {
                (CellChange::KnownEmpty, CellState::KnownEmpty) => continue,
                (CellChange::Put(p), CellState::Present(cur)) if **cur == *p => continue,
                (CellChange::KnownEmpty, _) => (None, ChangeType::Geometry),
                (CellChange::Put(p), cur) => (Some(Arc::new(p.clone())), classify(cur, p)),
            };
            changes.push(PlannedChange {
                layer: set.layer,
                level: w.level,
                brick: w.brick,
                payload,
                change,
            });
        }
    }
    if !conflicts.is_empty() {
        return Err(CommitError::Conflict(conflicts));
    }
    changes.sort_by_key(|c| (c.layer, c.level, c.brick));
    Ok(Plan { changes })
}

/// Classifies a payload change: only-material and only-attribute differences are reported as such.
fn classify(current: &CellState, new: &CellPayload) -> ChangeType {
    let (CellState::Present(old), CellPayload::Channels(n)) = (current, new) else {
        return ChangeType::Geometry;
    };
    let CellPayload::Channels(o) = &**old else {
        return ChangeType::Geometry;
    };
    let differing = differing_channels(o, n);
    if differing.is_empty() {
        ChangeType::Geometry
    } else if differing.iter().all(|c| *c == ChannelId::Material) {
        ChangeType::Material
    } else if differing
        .iter()
        .all(|c| matches!(c, ChannelId::Attribute(_)))
    {
        ChangeType::Attribute
    } else {
        ChangeType::Geometry
    }
}

fn differing_channels(a: &ChannelSet, b: &ChannelSet) -> Vec<ChannelId> {
    let ids: BTreeSet<ChannelId> = a.iter().chain(b.iter()).map(|(i, _)| i).collect();
    ids.into_iter()
        .filter(|i| a.channel(*i) != b.channel(*i))
        .collect()
}

/// Applies a validated plan, producing the new state and the outcome. Fails only on revision
/// overflow, before anything is published.
fn apply_plan(
    current: &AssetState,
    plan: Plan,
    actor: ActorKind,
    operation: OperationId,
) -> Result<(AssetState, CommitOutcome), CommitError> {
    let asset_rev = current.revision.next()?;
    let epoch = current.epoch;
    let asset_id = current.asset.id();
    let asset_token = RevisionToken::new(ResourceId::Asset(asset_id), epoch, asset_rev);
    let mut layer_revisions = current.layer_revisions.clone();
    let mut new_layers: BTreeMap<LayerId, CellStore> = BTreeMap::new();
    let mut layer_tokens = Vec::new();
    let mut cell_tokens = Vec::new();
    let mut dirty = Vec::new();
    for c in plan.changes {
        let store = match new_layers.entry(c.layer) {
            std::collections::btree_map::Entry::Occupied(e) => e.into_mut(),
            std::collections::btree_map::Entry::Vacant(e) => {
                let rev = layer_revisions[&c.layer].next()?;
                layer_revisions.insert(c.layer, rev);
                layer_tokens.push(RevisionToken::new(ResourceId::Layer(c.layer), epoch, rev));
                e.insert(current.store(c.layer).cloned().unwrap_or_default())
            }
        };
        let cell_rev = match store.revision(c.level, c.brick) {
            Some(r) => r.next()?,
            None => Revision::new(1),
        };
        store.commit_cell(c.level, c.brick, c.payload, cell_rev);
        let key = crate::key::CellKey::new(c.layer, c.level, c.brick)?;
        cell_tokens.push(RevisionToken::new(ResourceId::Cell(key), epoch, cell_rev));
        let min = c.brick.origin()?;
        let max = VoxelPos::new(
            min.x.checked_add(16).ok_or(CoreError::Overflow)?,
            min.y.checked_add(16).ok_or(CoreError::Overflow)?,
            min.z.checked_add(16).ok_or(CoreError::Overflow)?,
        );
        dirty.push(DirtyRegion {
            asset: asset_id,
            layer: c.layer,
            region: Region::new(c.level, min, max)?,
            change: c.change,
            source: asset_token,
            operation,
            actor,
        });
    }
    let layers: Vec<Layer> = current
        .asset
        .layers()
        .iter()
        .map(|l| match new_layers.remove(&l.id()) {
            Some(store) => {
                let data = match l.kind() {
                    LayerKind::Sdf => LayerData::Sdf(store),
                    _ => LayerData::Voxel(store),
                };
                Layer::new(l.id(), l.transform(), data)
            }
            None => l.clone(),
        })
        .collect();
    let asset = GeometryAsset::new(asset_id, layers)?;
    let state = AssetState {
        asset,
        epoch,
        revision: asset_rev,
        layer_revisions,
    };
    Ok((
        state,
        CommitOutcome {
            changed: true,
            asset_token,
            layer_tokens,
            cell_tokens,
            dirty,
        },
    ))
}
