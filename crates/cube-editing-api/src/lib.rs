//! Cube Editing API: public types and ports of layer 2 (PS-3).
//!
//! Contracts only, no behaviour: the editing service, geometry backends, observers and storage
//! are implemented elsewhere (Orchestration, backends, History, Persistence) against these
//! definitions. Depends on Core and Processing; never on an implementation.
//!
//! Contracts E-02..E-12 are Draft; this crate is their provisional implementation (see
//! `docs/production/editing-api-contract.md`). Changes to the public API are caught by
//! `tests/api_snapshot.rs` and need an accepted change request.

pub mod auth;
pub mod error;
pub mod ports;
pub mod storage;
pub mod types;

pub use auth::{ActorId, AuthorizationContext, Capability, Origin, Scope, ScopedRegion, SessionId};
pub use error::{ERROR_MODULE, EditingError};
pub use ports::{
    BackendCapabilities, EditingService, GeometryBackend, OverlayRead, TransactionObserver,
    ValidationReport,
};
pub use storage::{
    BaseProvider, BatchEncoder, BrickChange, ChangedBrick, Checkpoint, CheckpointId,
    CheckpointReason, DeltaId, DeltaStore, GeometryDelta, JournalEntry, JournalPhase, JournalSeq,
    NonBrickChange, NonBrickKind, OperationJournal, PayloadRef, SnapshotRef, SnapshotStore,
};
pub use types::{
    CommitPolicy, ConflictResult, ConflictScope, ConversionPolicy, EditMode, EditingOperation,
    EditingResult, KindId, OperationStatus, Parameters, RollbackPolicy, SchemaVersion, Success,
    Suggestion, TransactionHandle, TransactionId, TransactionSpec, TransactionState,
    TransactionSummary, kinds,
};
