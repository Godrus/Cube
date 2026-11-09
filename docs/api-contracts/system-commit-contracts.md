# Cube — System Commit Contracts (Draft)

**Status:** DRAFT 0.1.0-draft, **Proposed text, nothing approved.** Not part of the 45 numbered contracts and not assigned to a tier; P0/P1 are untouched (AOD-18). The contracts below **propose extensions** to C-17 OperationId, the commit primitive and the RevisionToken semantics; those P0 contracts are not edited here. The placement of the new port is the subject of **ACR-004 (Pending)**.
**Basis:** AOD-15, AOD-16, AOD-20 (system generation gets an OperationId and takes part in revision/commit semantics; it is not an ordinary Editing-API operation and not in ordinary undo/redo; no bypass of public Core ports), `revision-and-snapshot-semantics.md`, `editing-contracts.md` (E-02 AuthorizationContext).
Field order as in `worldgen-contracts.md`. No language-specific signatures.

## 0. Role

Exactly three callers may commit to the authoritative asset: the Editing service (user operations), the WorldGen Materialiser (generated data) and the persistence loader (recovery). The Editing service uses the existing commit path; the other two use the **system-commit port** defined here. Nothing else may write (call-site audit test).

## SC-01 OperationOrigin = ActorKind on OperationId (extension of OperationId, proposal; ACR-004 part C)

* **Purpose:** distinguish who caused a commit without changing OperationId's identity role.
* **Model:** `OperationId` as in C-17 plus the origin, expressed as `ActorKind ∈ {User, System, Recovery}` (recommended in `docs/decisions/acr-004-owner-review.md`, requires owner approval) (carried with the id, not inside its numeric part; whether it is a field of the id or of the commit request is open).
* **Rules:** allocated once per submitted commit (OD-R8 applies); the origin never changes; `Recovery` commits reuse the **original OperationId** of the journal record they replay.
* **Open:** id format (OD-R8), whether the origin is part of the id.

## SC-02 ActorKind

* **Model:** `User` (a player or designer through the Editing API; authorisation per E-02), `System` (WorldGen Materialiser and server-internal automation), `Recovery` (persistence loader replaying a snapshot and journal).
* **Rules:** ActorKind is derived by the server from the authenticated path, never supplied by a client. The `Origin` field of E-02 (LocalClient | Server | System) is a related notion; mapping is open. `LocalClient` gains **no extra rights** in the embedded server (`server-runtime-contracts.md`, SR-06).
* **Errors:** `PermissionDenied` when a User actor targets the system-commit port.

## SC-03 CommitTarget

* **Model:** `AssetId, LayerId, set of BrickKeys | GeometryRegion` as in C-01, C-02, C-06, C-07. A system commit targets whole bricks or chunks; it may not target an arbitrary voxel set mixed with user-owned data without the precedence rule (open: single layer or C2 overlay).

## SC-04 ExpectedRevision

* **Purpose:** optimistic concurrency for system commits.
* **Model:** `ExpectedTokens` as in the RevisionToken proposal; a system commit carries the token it observed (or `None` for first materialisation of an unloaded chunk).
* **Rules:** a mismatch gives `RevisionMismatch`; the Materialiser re-reads and retries. Materialisation of a chunk that was meanwhile edited by a user is rejected and reconciled by the service (user data wins; never overwritten).
* **Revision model (OAM-10, `Proposed / Requires Owner Approval`):** **recommended R-A**: a system commit advances the same asset and layer scopes (and brick revisions) as any commit and is identified by `ActorKind::System`; no separate generation epoch (R-B) and no separate domains for base and overrides (R-C) unless necessity is demonstrated. The existing `Epoch` keeps its meaning (re-creation or recovery). User edits use brick or region stamps as expected tokens so that streaming does not cause spurious conflicts. **Open (B-6):** RevisionToken width and commit algorithm.

## SC-05 SystemCommitRequest / CommitResult

* **Model:** `SystemCommitRequest {OperationId, Origin, ActorKind, Target, ExpectedRevision, Payload (GeneratedChunk | RecoveredState), AuditMetadata}`; `CommitResult {Status ∈ {Applied, AlreadyApplied, Rejected}, NewTokens, DirtyRegions, Error?}`.
* **Rules:** validate, then apply atomically at brick/chunk granularity under the existing commit rule (short lock); no half-applied result is visible; `Rejected` leaves state unchanged.
* **Tests:** atomicity (property), snapshot isolation, negative tests per error code.

## SC-06 DirtyRegions

* **Model:** the commit returns the same `DirtyRegion` structure as C-18 with a `ChangeType` that identifies generated or restored data (new values proposed: `Generated`, `Restored`; an addition to the ChangeType set, to be reviewed with C-18).
* **Rules:** renderers and observers treat the regions as normal dirty input; they must not use them to infer a user edit.

## SC-07 Idempotency

* **Rules:** the same `OperationId` with the same content hash returns `AlreadyApplied` and changes nothing; the same id with a different content hash is `OperationConflict` and is a defect report. Replay of a journal (`Recovery`) is idempotent by this rule.
* **Tests:** double materialise, double replay, crash and restart mid-replay.

## SC-08 AuditMetadata

* **Model:** `ActorKind, OperationId, GeneratorVersion?, SeedRef?, ContentHash, ServerTick, ServerInstanceId`.
* **Rules:** an audit trail, not history: system commits are not offered to undo. Retention is open (whether system commits are logged durably at all; generation is regenerable and may be logged only in a bounded diagnostic log).

## SC-09 UndoPolicy

* **Rules:** `System` and `Recovery` commits are **never** undoable and never enter ordinary undo/redo (AOD-20). A user undo restores the *effective previous voxel* (generated or earlier edit), not a stored absolute value; undoing across a chunk whose generator version changed is a controlled error.
* **Tests:** generation does not appear in history; undo of an edit over generated terrain restores generated terrain.

## SC-10 SystemCommitPort

* **Model:** `Materialise(chunk)`, `Evict(chunk)`, `Restore(state)` (the last for the loader), each taking a `SystemCommitRequest`.
* **Rules:** callable only by the Materialiser and the persistence loader (authorised by ActorKind and a build-time dependency rule); never exposed to the network; `Evict` removes generated data only and never user changes; every call is atomic and idempotent.
* **Tests:** call-site audit of the three writers; a client cannot reach the port; WorldGen sees no Core internals beyond the port.
* **Open:** signature, error set, thread-safety, whether `Evict` is a commit at all (RevisionToken effect).

## Not approved

SC-01…SC-10 are Draft. OperationId origin representation, the revision effect (B-6), the ChangeType additions, the audit retention and the port signature are open. **ACR-004 is Pending** (owner review package: `docs/decisions/acr-004-owner-review.md`); until it is decided no P0 contract is edited and no production Core code is written.
