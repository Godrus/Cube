# Editing API: provisional implementation record (PS-3)

**Status:** Owner-Approved for the PS-3 scope (GOV-09, DEC-082..DEC-086); provisional parts are not Frozen. `crates/cube-editing-api` implements E-02...E-07, E-09...E-12 and the storage ports as plain types and traits (contracts only, no behaviour). The contracts stay Draft until GOV-09 (owner approval); nothing here is Frozen. API changes are caught by `crates/cube-editing-api/tests/api_snapshot.rs` (regenerate with `UPDATE_API_SNAPSHOT=1`, state the change request in the commit).

## Task coverage

| Task | Content | Files |
|---|---|---|
| API-01 | operations, transactions, results, conflicts (E-03..E-06), error enum with stable codes 4001..4015 | `src/types/`, `src/error.rs` |
| API-02 | `EditingService` (E-11), `GeometryBackend` + `BackendCapabilities` (E-07), `TransactionObserver` (E-12) | `src/ports/` |
| API-03 | `AuthorizationContext` (E-02); origin to `ActorKind` mapping; no extra rights from `LocalClient` | `src/auth.rs` |
| API-04 | `GeometryDelta` (E-09), `Checkpoint` (E-10), `DeltaStore`, `SnapshotStore`, `OperationJournal`, `BaseProvider` (uses `WorldSeed`/`GeneratorVersion`) | `src/storage.rs` |
| API-05 | public-API snapshot test | `tests/api_snapshot.rs`, `api-snapshot.txt` |

Rust naming: the contract's `IEditingService` is `EditingService`, `IGeometryBackend` is `GeometryBackend`, and so on.

## Provisional resolutions of "Open" items (reversible, decided by delegation)

1. Backends receive read-only views (`OverlayRead`, optional Processing `Snapshot`) and **return** `Vec<WriteSet>`; Orchestration stages and commits them. A backend cannot write by construction. (E-07 open: overlay mutation.)
2. Parameter schemas stay with the kind owner: `Parameters` is an opaque, versioned byte block; kind ids below 1000 are reserved for the nine kinds the contract names. (E-03 open: kind registry.)
3. The facade is synchronous; async job handles are not part of the minimal contract. (E-11 open.)
4. `CommitPolicy::Atomic` and `RollbackPolicy::Unchanged` are the only policies; the enums can grow. (E-04.)
5. Conflict reports carry `ResourceId`s and paired expected/current tokens; resolution stays "reject and report" (OD-R3).
6. Payload bytes, delta encoding and serialisation are opaque (`PayloadRef`); no format is implied. (E-09 open.)
7. The journal assigns gapless sequence numbers from 1 and has no owner yet; only its shape is fixed.

## Owner decisions recorded (GOV-09)

DEC-082 approves the contracts and the seven resolutions; DEC-083 names Server/Authorization as owner of the permission schema; DEC-084 fixes one stroke = one transaction and no nesting; DEC-085 defers delta encoding and checkpoint frequency to PS-10; DEC-086 fixes journal ownership (Persistence) and observer delivery rules.

## Contract blocker CB-1 (journal and Core commit are not one atomic action)

Core's commit and a durable journal write cannot be made atomic with the current interfaces. Safe boundary (additive API change, change request of PS-3 close): `JournalPhase` (`Intent` before the commit, `Confirmed` after, `Aborted` if the commit did not happen) on `JournalEntry`, and `EditingResult::AppliedUnconfirmed` for "applied, confirmation not durable" (never reported as success). A retry with the same `OperationId` completes the confirmation without applying twice. An `Intent` without a later record is resolved at recovery against Core's commit log; Persistence defines that procedure in PS-10. Crash durability is **not** claimed before PS-10.
