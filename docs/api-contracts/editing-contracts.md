# Cube — Editing API & Contracts

**Status:** DRAFT 0.1 — specified for approval; none approved.
**Owner:** Main Agent / Integration Owner. Basis: AU-1.0 §3.2, §13, §14, §20, §23; Spec §5–§6, §9, §12.
Conventions: `README.md`. Field order: Purpose · Model · Ownership · Lifetime · Access · Revision · Dirty · Threading · Cancel · Errors · Serialization · Compat · Tests · Open.

## 0. Role of this layer

Layer 2. Depends on Core and Processing. Contains **only** contracts and the public facade interface; implementations live in Orchestration, Backends, History, Persistence, Networking. It is also the place for shared abstractions that break cycles (AU-1.0 §3.2):

| Port | Defined here | Implemented by | Consumed by |
|---|---|---|---|
| `IEditingService` (Begin/Execute/Commit/Rollback/Cancel/Status) | yes | Orchestration (A) | Tools, Composite, Resolution, History (undo), Networking |
| `IGeometryBackend` + `BackendCapabilities` | yes | Voxel/SDF/Mesh | Orchestration (registry), Tools via capability query |
| `ITransactionObserver` (post-commit journal sink) | yes | History (G) | Orchestration |
| `IDeltaStore` / `ISnapshotStore` (durable storage port) | yes | Persistence (H) | History (G) |
| `IOperationJournal` (confirmed operation journal) | yes (shape only) | Open (no owner) | Networking |

History and Persistence therefore never import each other; both depend on this layer's data contracts (GeometryDelta, Checkpoint) and abstractions.

---

## E-01 OperationId — see C-17 (Core foundation).

## E-02 AuthorizationContext

- **Purpose:** who is acting and with what rights, evaluated server-side.
- **Model:** `ActorId, Roles/Capabilities[], Scope (assets/regions allowed), Origin (LocalClient | Server | System), SessionId`.
- **Ownership:** produced by the Networking/session layer (outside Geometry); Editing API treats it as an opaque, immutable input.
- **Lifetime:** per transaction. **Access:** read-only. **Revision:** none. **Dirty:** none.
- **Threading:** value-safe. **Errors:** `PermissionDenied`. **Serialization:** not persisted with geometry; journaled as `ActorId`.
- **Rules:** authorization is checked before revision validation; undo/redo respect authorization (Spec §12.2).
- **Tests:** denial leaves no state; undo of another actor's edit.
- **Open:** permission model, region-level ACLs, ownership of the permission schema (gameplay/server).

## E-03 EditingOperation (command)

- **Purpose:** declarative intent of one edit.
- **Model:** `OperationId, Kind (extensible: SetMaterial, AddGeometry, RemoveGeometry, TransformGeometry, Boolean*, Sculpt*, Resample, Bake, Composite*...), TargetAssetId/LayerIds, Parameters (typed per Kind), ReadRegion, WriteRegion, ConversionPolicy?, Mode (Destructive | NonDestructive)`.
- **Ownership:** value; Kind schemas owned by the module that defines the Kind (Tools own Sculpt/Boolean schemas, backends own low-level kinds), registered with the Editing API.
- **Lifetime:** immutable once submitted; copied into History.
- **Revision:** supplies expected tokens (via the transaction). **Dirty:** declared WriteRegion is an upper bound; actual DirtyRegions reported by commit.
- **Errors:** `UnsupportedGeometryOperation`, `IncompatibleGeometryTypes`, `ConversionRequired`, `InvalidResolution`.
- **Serialization:** stored in History and networked; Kind id and schema versions stable. **Compat:** unknown Kind ⇒ controlled error.
- **Tests:** schema round-trip; mixed-type rejection without ConversionPolicy.
- **Open:** Kind extension registry; two-level (low-level API vs transactional) exposure rule.

## E-04 EditingTransaction

- **Purpose:** atomic unit of change.
- **Model:** `TransactionId, OperationId(s), Targets, ReadRegions, WriteRegions, ExpectedTokens, OperationSequence, CommitPolicy, RollbackPolicy, AuthorizationContext, State (Open | Committing | Committed | RolledBack | Failed | Cancelled)`.
- **Ownership:** Orchestration owns the live transaction; callers hold a handle.
- **Lifetime:** Begin → terminal state; released after notification of observers.
- **Access:** operations execute against a private overlay (COW) – no authoritative mutation before commit.
- **Revision:** commit protocol in semantics §4.1; rollback/cancel change no revision.
- **Dirty:** collected from overlay, emitted atomically at commit.
- **Threading:** single logical writer per transaction; many transactions concurrent when regions are disjoint.
- **Cancel:** supported before `Committing`; after that, the commit completes or fails as a unit.
- **Errors:** `RevisionConflict`, `TransactionConflict`, `PermissionDenied`, `ProcessingCancelled`, `ProcessingFailed`. Every operation declares failure behaviour (unchanged / rollback / recoverable partial / external reconciliation) per Agent Spec §9.
- **Serialization:** journaled form only. **Tests:** atomicity property (failed ⇒ unchanged), disjoint-region concurrency, nested/grouped (Open).
- **Open:** nested transactions; group semantics for one sculpt stroke (one transaction per stroke vs per dab); long-running (async) transaction representation.

## E-05 EditingResult (GeometryOperationResult)

- **Model:** variant `Success{OperationId, NewTokens, DirtyRegions, ProducedIds, DeltaRef?} | ValidationFailure | Conflict(ConflictResult) | Unsupported | ConversionRequired | ResourceFailure | Cancelled`.
- **Rules:** partial results exist only if an operation's contract declares one explicitly. Result contains no memory references.
- **Ownership:** value returned to caller. **Revision:** reports authoritative new tokens. **Threading:** value-safe.
- **Tests:** each variant produced; no partial result without declaration.

## E-06 ConflictResult

- **Model:** `Scope (Brick|Layer|Asset|Lock), ConflictingKeys[], ExpectedTokens, CurrentTokens, HoldingOperationId? (for locks), Suggestion (Retry | Rebase | Abort)`.
- **Rules:** informative only; resolution policy default "reject and report" (OD-R3).
- **Tests:** contains exactly the conflicting keys; no leakage of unauthorised data.

## E-07 IGeometryBackend / BackendCapabilities

- **Purpose:** the only way Orchestration reaches representation-specific logic.
- **Model:** `SupportedLayerType`, `Capabilities (set of OperationKinds, NativeBoolean?, Sculpt tool set, Resample support, Validate)`, `Apply(operation, overlay, snapshot?) → Result<WriteSet, Error>`, `Validate(layer region) → ValidationReport`.
- **Rules:** backends never call Tools or Orchestration; mutation only via the overlay passed in; must report DirtyRegions for actual changes; no implicit conversion.
- **Threading:** `Apply` may run on workers against snapshots; overlay is private.
- **Tests:** conformance suite run against every backend (owned by QA).
- **Open:** granularity of the capability vocabulary; whether Sculpt/Boolean dispatch through `Apply` or dedicated sub-interfaces.

## E-08 ModifierStack

- **Purpose:** non-destructive editing state.
- **Model:** `LayerId, SourceRef, Modifiers[ModifierDescriptor], EvaluationRevision`, Evaluated Geometry is derived data (Processing slot).
- **Ownership:** data model in Editing contracts, stored on the Layer (reference from C-02). **Evaluation provider:** Resolution/Backends per modifier kind; scheduling by Processing. Bake is a transaction (Spec §6.3).
- **Revision:** modifier change increments Layer revision; EvaluationRevision = DependencyRevision of the evaluated slot.
- **Dirty:** Geometry (derived). **Errors:** `UnsupportedGeometryOperation` for kinds without non-destructive support.
- **Serialization:** manifest + stack payload. **Tests:** bake undoable; invalidation on modifier edit.
- **Open:** **who evaluates modifiers** (no agent owns Evaluation today) — see audit report O-2; modifier kind registry.

## E-09 GeometryDelta

- **Purpose:** record of actual data changes, shared by History and Persistence.
- **Model:** `DeltaId, OperationId, AssetId, LayerId, PreTokens, PostTokens, ChangedBricks[(BrickKey, BeforeRef | AfterRef | PatchPayload)], NonBrickChanges[(transform, resolution, palette, modifiers, junction, composite)], DirtyRegions`.
- **Ownership:** value built from the commit; payload stored via `IDeltaStore`. **History stores delta metadata and holds no brick storage of its own**; payload bytes live in the durable store or Core COW versions.
- **Lifetime:** retention per History limits; cannot be freed while referenced by Checkpoint/journal.
- **Revision:** PreTokens/PostTokens drive undo validation. **Serialization:** versioned (Delta Serialization P-03).
- **Errors:** `PersistenceFailure`. **Tests:** apply(inverse(delta)) restores bit-exact data for the representation; patch composition associativity.
- **Open:** delta encoding (brick-diff vs full-brick vs command replay); size limits.

## E-10 Checkpoint

- **Purpose:** restoration accelerator and durable recovery point.
- **Model:** `CheckpointId, AssetToken, LayerTokens, SnapshotRef (COW or durable), LastDeltaId, Epoch, Reason`.
- **Ownership:** created by History (policy) via the snapshot service; durable form owned by Persistence behind `ISnapshotStore`.
- **Lifetime:** by History limits; durable checkpoints protected until superseded and verified.
- **Revision:** restoring a checkpoint produces new forward revisions and a new Epoch where history older than live state is discarded (see semantics §6).
- **Tests:** restore equals replay; memory cap respected. **Open:** snapshot frequency policy.

## E-11 IEditingService (facade)

- **Model (conceptual):** `BeginTransaction(context) → handle`, `Execute(handle, operation) → Result`, `Commit(handle)`, `Rollback(handle)`, `Cancel(operationId)`, `GetStatus(operationId)`; async variants return `AsyncJobHandle` (= ProcessingTask handle).
- **Rules:** Low-level API cannot bypass Dirty Tracking, Revision management, or Dependency Graph (Spec §5.1). Exact signatures are an Open Decision.

## E-12 ITransactionObserver

- **Model:** `OnCommitted(TransactionSummary, GeometryDelta)`, `OnRolledBack`. Observers cannot veto or mutate. Delivery ordered per Asset, after the commit gate is released, durable-before-ack ordering is Open.
- **Implemented by:** History; optionally Networking/Journal. **Tests:** exactly-once, ordered delivery; observer failure does not undo commit (reported, logged).

## E-13 Networking contracts (boundary only; AU-1.0 §20)

- **Model of a network operation:** `OperationId, AuthorizationContext, ExpectedTokens, Payload (EditingOperation), ClientPredictionRef`; server reply: `EditingResult` + authoritative tokens + `ConfirmedJournalEntry`.
- **Rules:** data-based sync (operations/deltas), never render meshes as truth; reconnect replay by OperationId; reconciliation per Spec §13.5.
- **Owner:** **unassigned** (Networking Adapter has no owner in AU-1.0 §2) — audit report O-3. Protocol/transport not approved.

## RS-1: retry safety of operation ids (Orchestration requirement, L-COMP-1)

Status: provisional, like the editing contracts. It does **not** change the Core `OperationId` model (C-17: opaque, equality only, format open under OD-R8); it states what Orchestration needs from the *source* of ids to answer old retries safely with bounded memory and disk.

* The sequences of one scope are allocated by the authoritative side and increase strictly. Arbitrary random identifiers are not assumed to be ordered.
* Compaction persists, per scope, the highest folded confirmed sequence (the *retry floor*, in the manifest). Cost is bounded by the number of scopes (cap 65 536), not by the number of operations.
* An id at or below its scope's floor that the journal window no longer shows is refused (`expired`), never applied again, never judged "new"; the same holds for a changed payload. A source that breaks the rule (a lower, unused sequence) is refused too: fail closed.
* The caller restores the floors (`OrchestrationService::restore_retry_floors`) before accepting operations, then calls `recover_idempotency`.
* If a future OD-R8 decision chooses unordered ids, this requirement must be revisited (floors would then need a per-id structure).
