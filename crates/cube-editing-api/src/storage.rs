//! Storage and recovery contracts (API-04, E-09 GeometryDelta, E-10 Checkpoint, ports of the
//! durable store, the operation journal and the base provider).
//!
//! History and Persistence never import each other: both depend on these data contracts and
//! abstractions. Recovery needs no WorldGen dependency: the base provider is a port that the
//! composition root fills. Payload bytes and encodings are opaque here (delta encoding and the
//! serialisation format are open decisions).

use cube_core::{
    AssetId, AssetView, BrickCoord, CellChange, CellKey, DirtyRegion, Epoch, GeneratorVersion,
    LayerId, OperationId, RevisionToken, WorldSeed,
};

use crate::auth::ActorId;
use crate::error::EditingError;

/// Identifier of a delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeltaId(pub u64);

/// Identifier of a checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CheckpointId(pub u64);

/// Reference to payload bytes held by a store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PayloadRef(pub u64);

/// Sequence number of a journal entry, starting at 1, gapless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JournalSeq(pub u64);

/// What is stored for a changed brick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrickChange {
    /// The brick content before the change.
    Before(PayloadRef),
    /// The brick content after the change.
    After(PayloadRef),
    /// A patch between before and after.
    Patch(PayloadRef),
}

/// One changed brick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChangedBrick {
    /// Which brick.
    pub key: CellKey,
    /// How its change is stored.
    pub change: BrickChange,
}

/// Changes that are not brick data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NonBrickKind {
    /// Layer transform.
    Transform,
    /// Resolution.
    Resolution,
    /// Palette.
    Palette,
    /// Modifier stack.
    Modifiers,
    /// Junction.
    Junction,
    /// Composite.
    Composite,
}

/// A non-brick change with its stored before and after state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonBrickChange {
    /// What changed.
    pub kind: NonBrickKind,
    /// State before.
    pub before: PayloadRef,
    /// State after.
    pub after: PayloadRef,
}

/// Record of the actual data changes of one commit, shared by History and Persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryDelta {
    /// Identity of the delta.
    pub id: DeltaId,
    /// Operation that caused it.
    pub operation: OperationId,
    /// Asset.
    pub asset: AssetId,
    /// Layer.
    pub layer: LayerId,
    /// Tokens before the commit (drive undo validation).
    pub pre_tokens: Vec<RevisionToken>,
    /// Tokens after the commit.
    pub post_tokens: Vec<RevisionToken>,
    /// Bricks that changed.
    pub changed_bricks: Vec<ChangedBrick>,
    /// Non-brick changes.
    pub non_brick: Vec<NonBrickChange>,
    /// Regions that actually changed.
    pub dirty: Vec<DirtyRegion>,
}

/// Why a checkpoint was taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckpointReason {
    /// By History policy (frequency).
    Policy,
    /// Compaction of the journal.
    Compaction,
    /// Requested explicitly.
    Requested,
}

/// Where the checkpointed state lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotRef {
    /// A copy-on-write version held in memory.
    Cow(u64),
    /// A verified durable snapshot.
    Durable(PayloadRef),
}

/// Restoration accelerator and durable recovery point (E-10). Never the only source of truth
/// before it is verified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    /// Identity.
    pub id: CheckpointId,
    /// Asset token at the checkpoint.
    pub asset_token: RevisionToken,
    /// Layer tokens at the checkpoint.
    pub layer_tokens: Vec<RevisionToken>,
    /// Where the state is.
    pub snapshot: SnapshotRef,
    /// Last delta contained, if any.
    pub last_delta: Option<DeltaId>,
    /// Epoch of the state.
    pub epoch: Epoch,
    /// Why it was taken.
    pub reason: CheckpointReason,
}

/// Phase of a journal record. A durable operation is journalled write-ahead: an `Intent` before
/// the Core commit, then `Confirmed` (with the post tokens) after it, or `Aborted` if the commit
/// did not happen. An `Intent` without a later record is resolved at recovery by comparing it
/// with Core (the operation id is in Core's commit log) - Persistence defines that procedure
/// (PS-10). Recovery never reuses an operation id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JournalPhase {
    /// The operation is about to be committed.
    Intent,
    /// The commit happened.
    Confirmed,
    /// The commit did not happen.
    Aborted,
}

/// One record of the operation journal (the shape that networking replays).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntry {
    /// Position in the journal.
    pub seq: JournalSeq,
    /// Phase of the record.
    pub phase: JournalPhase,
    /// The operation.
    pub operation: OperationId,
    /// Who acted.
    pub actor: ActorId,
    /// Asset changed.
    pub asset: AssetId,
    /// Authoritative tokens after the commit.
    pub post_tokens: Vec<RevisionToken>,
    /// Delta recorded for it, if any.
    pub delta: Option<DeltaId>,
    /// Every operation the record covers, each with a stable fingerprint of its content (see
    /// `EditingOperation::fingerprint`). A transaction commits under its first operation id
    /// (`operation`), but a retry may name any member; recovery of the idempotency table needs
    /// all of them, and the fingerprint tells a true retry from a reused id.
    pub members: Vec<(OperationId, u64)>,
}

/// Durable store of deltas and their payloads (implemented by Persistence, used by History).
pub trait DeltaStore: Send + Sync {
    /// Stores a payload and returns its reference.
    fn put_payload(&self, bytes: &[u8]) -> Result<PayloadRef, EditingError>;
    /// Reads a payload.
    fn payload(&self, r: PayloadRef) -> Result<Vec<u8>, EditingError>;
    /// Stores a delta. Durable before it returns `Ok`.
    fn put_delta(&self, delta: &GeometryDelta) -> Result<(), EditingError>;
    /// Reads a delta.
    fn delta(&self, id: DeltaId) -> Result<GeometryDelta, EditingError>;
}

/// Durable store of checkpoints (implemented by Persistence, used by History).
pub trait SnapshotStore: Send + Sync {
    /// Stores a checkpoint with the full state it describes. The checkpoint becomes visible only
    /// when it is complete and verified; a crash leaves the previous one in place.
    fn put_checkpoint(
        &self,
        checkpoint: &Checkpoint,
        state: &AssetView,
    ) -> Result<(), EditingError>;
    /// Loads a checkpoint and its state.
    fn checkpoint(&self, id: CheckpointId) -> Result<(Checkpoint, AssetView), EditingError>;
    /// The newest verified checkpoint of an asset, if any.
    fn latest(&self, asset: AssetId) -> Result<Option<Checkpoint>, EditingError>;
}

/// Confirmed operation journal (shape only; no owner is assigned yet).
pub trait OperationJournal: Send + Sync {
    /// Appends an entry. The store assigns the sequence number and returns it.
    fn append(&self, entry: JournalEntry) -> Result<JournalSeq, EditingError>;
    /// Reads up to `limit` entries with `seq >= from`, in order.
    fn read_from(&self, from: JournalSeq, limit: usize) -> Result<Vec<JournalEntry>, EditingError>;
}

/// Supplies the generated (untouched) content of a brick to recovery: the base of
/// "base generation, then snapshot, then journal tail". Implemented by WorldGen; recovery depends
/// only on this port.
pub trait BaseProvider: Send + Sync {
    /// The content of the brick `(layer, level, brick)` of `asset` as generated from `seed` by
    /// `version`. Deterministic: equal arguments give equal results.
    fn base_brick(
        &self,
        seed: WorldSeed,
        version: GeneratorVersion,
        asset: AssetId,
        layer: LayerId,
        level: i8,
        brick: BrickCoord,
    ) -> Result<CellChange, EditingError>;
}
