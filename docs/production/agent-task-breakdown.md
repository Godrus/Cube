# Cube Agent Task Breakdown v2 (proposal)

**Status:** **Proposed**; no task is authorised to start by this document. The tables below are **generated from the same data file as the ADG** (`module-boundaries.md`, part C), so ids, owners, stages and scope cannot diverge. **Dependencies are not repeated here:** the column "Depends" is replaced by the ADG; look a task up by id in part C (start-after `S`, merge-after `I`). A task with no entry in the ADG does not exist.
**Ownership rules:** (1) each crate has exactly one owner (machine-checked); (2) a task touches only files of its own crate; a change to another crate's API is a change request to that owner; (3) tests inside a crate belong to the crate owner, cross-crate suites live in `cube-integration-tests` (Agent I); (4) tasks owned by `UNASSIGNED` modules cannot start before the owner decision GOV-03 names the owner (rule R-UNOWNED); (5) a task that needs a contract in status Proposed may only implement it behind an extension point and cannot merge as production unless the contract is Owner-Approved (or registered Provisional).
**Common acceptance for every task:** the crate's tests green in CI; `archcheck` green; public-API snapshot unchanged or an accepted change request; no new dependency without a licence check; PR per module (`integration-and-qa-plan.md`).
**Columns:** Scope · Files/modules · Public API · Required tests · Deliverables · Acceptance. *Forbidden scope* is given per owner and applies to every row of that owner.

## 1. Project owner
**Crate / area:** decisions. **Acceptance authority:** project owner. **Forbidden scope:** delegating decisions to agents implicitly.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| DEV-01 | PS-0 | Acquire physical Android ARM64 devices and macOS/Xcode access (does not gate PS-1..PS-13) | n/a | none | n/a | device inventory | recorded; gates only DG-1..DG-3 |
| GOV-01 | PS-0 | Decide ACR-001/002/003, ACR-004 part A (Core prerequisites) and part D principle (OAM-10 revision model); OD-R1, OD-R2, OD-R6, D-R8 (OperationId format), D-K, D-T, D-C, D-0 (P0-alpha scope); D-E (toolchain, execution model, dependency licence allow-list); the branch option for the docs-only baseline (GOV-04) | docs/decisions/decision-registry.md | none | review checklist | decision-registry entries with confirmation source | every PS-1 prerequisite decision is Owner-Approved, Rejected or explicitly deferred with an extension point |
| GOV-02 | PS-0 | Owner-Approve P0-alpha; record Provisional Contracts PC-1 with assumptions and revalidation gate RG-1; adopt the Contract Status Model | docs/api-contracts/README.md (status column), decision-registry.md | none | n/a | registry entries; contract status updates made by the Integration Owner on the owner record | P0-alpha status Owner-Approved; PC-1 status Provisional with all mandatory fields |
| GOV-03 | PS-0 | Decide owners for UNASSIGNED modules needed by the next stages (WorldGen, Networking Adapter, Server Runtime, Client Runtime, Engine Adapter) and for Import/Export, Derived Data Providers (O-1), Modifier Evaluation (O-2), or defer each in writing | docs/production/module-boundaries.md (ownership table) | none | n/a | ownership table updated by the Integration Owner from the owner record | every module needed by PS-1..PS-5 has a named owner; others deferred with the stage that needs them |
| GOV-07 | PS-2 | Owner-Approve PR-01..PR-08 after the Geometry Processing Specification is supplied (RG-2) | decision-registry.md | none | n/a | registry entry | PR-01..PR-08 Owner-Approved or an owner-recorded alternative |
| GOV-16 | PS-2 | Owner-Approve C-05 GeometryBounds | decision-registry.md | none | n/a | registry entry | Owner-Approved or amended |
| GOV-24 | PS-2 | Decide OD-R4 (snapshot lifetime and caps) | decision-registry.md | none | n/a | registry entry | Owner-Approved or amended |
| GOV-09 | PS-3 | Owner-Approve E-02..E-12 (P1 Editing slice) and the storage ports | decision-registry.md | none | n/a | registry entry | Owner-Approved or amended |
| GOV-08 | PS-5 | Record the result of revalidation gate RG-1: Voxel Architecture compared with PC-1 (C-07, C-08, C-09) and C-10…C-12; required before any Frozen status of those contracts | decision-registry.md | none | n/a | registry entry | PC-1 confirmed, amended or replaced; result recorded |
| GOV-17 | PS-5 | Owner-Approve C-10 MaterialMapping (after RG-1) | decision-registry.md | none | n/a | registry entry | Owner-Approved or amended |
| GOV-20 | PS-6 | Owner-Approve tool-level contracts for Sculpt/Brush and BooleanPolicy (B-7) | decision-registry.md | none | n/a | registry entry | Owner-Approved before tool implementations merge |
| GOV-25 | PS-7 | Decide OD-R5 (undo when later edits touched the same cells) | decision-registry.md | none | n/a | registry entry | Owner-Approved or amended |
| GOV-19 | PS-8 | Owner-Approve C-13, C-14, C-15 (junction, composite part, assembly graph; after ACR-003) | decision-registry.md | none | n/a | registry entry | Owner-Approved or amended |
| GOV-23 | PS-8 | Owner-Approve composite-operation contracts and resample/refine contracts (B-7) | decision-registry.md | none | n/a | registry entry | Owner-Approved before COMP and RES implementations merge |
| GOV-18 | PS-9 | Owner-Approve C-11 ResolutionProfile and C-12 RefinementRegion (after RG-1) | decision-registry.md | none | n/a | registry entry | Owner-Approved or amended |
| GOV-10 | PS-10 | Owner approves the persistence format after SPK-PERS and PERS-05 results and after the Voxel Architecture revalidation (RG-1) | decision-registry.md | none | n/a | registry entry | format Owner-Approved or amended |
| GOV-11 | PS-10 | Decide ACR-004 parts B (WorldGen integration), C (Persistence recovery) and E (removal of legacy direct writes); OAM-09 (C1/C2) | decision-registry.md | none | n/a | registry entry | each part Owner-Approved, Rejected or deferred |
| GOV-21 | PS-11 | Owner-Approve the WorldGen contracts WG-01…WG-10 | decision-registry.md | none | n/a | registry entry | Owner-Approved before the 11B tasks merge |
| GOV-12 | PS-13 | Owner chooses the external transport after SPK-NET | decision-registry.md | none | n/a | registry entry | Owner-Approved or amended |
| GOV-22 | PS-13 | Owner-Approve the server-runtime contracts SR-01…SR-10 and the protocol/session type contracts | decision-registry.md | none | n/a | registry entry | Owner-Approved before NET-02 and SRV-01 merge |
| GOV-13 | PS-14 | Owner records the default renderer after DG-1 | decision-registry.md | none | n/a | registry entry | recorded (AOD-10 stays until then) |
| GOV-15 | PS-14 | Decision to Freeze PC-1 (C-07, C-08, C-09): allowed only after the RG-1 revalidation record | decision-registry.md | none | n/a | registry entry | no Provisional contract is Frozen without GOV-08 |

## 2. Main Agent / Integration Owner
**Crate / area:** cube-editing-api, xtask, composition, reviews. **Acceptance authority:** project owner (contracts); Integration Owner (engineering). **Forbidden scope:** implementing behaviour in cube-editing-api; approving contracts.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| GOV-04 | PS-0 | Docs-only baseline on master (docs branch must not carry spike code); owner chooses the option | master (docs only) | none | n/a | baseline branch/PR | no file of the spike code list is on master |
| GOV-05 | PS-0 | CI SPECIFICATION document (checks, required statuses, tool pins) - a specification, not a running CI | docs/production/integration-and-qa-plan.md section 5 | none | n/a | Owner-Approved CI specification | specification reviewed; every check has an owner and a pass condition |
| XT-01 | PS-0 | archcheck v0: crate graph equals the ADG module edges; use-level scan for forbidden imports | xtask/src/archcheck.rs | CLI | seeded-violation self-tests | tool | catches every seeded violation; registered as a required status check |
| INT-01 | PS-1 | Record contract status Implemented for the P0-alpha contracts; PC-1 contracts stay at Provisional approval (status model: approval axis unchanged) | docs/api-contracts/README.md (status column), decision-registry.md | none | n/a | status record by the Integration Owner | Implemented is recorded only with CI evidence; no contract is set Frozen |
| XT-02 | PS-1 | archcheck v1: mutation-boundary rules (only allowed crates reference the handles), no history/undo patterns in adapters, API snapshots | xtask/src/archcheck.rs | CLI | seeded violations incl. a worldgen crate naming UserCommit | tool | every seeded direct-write violation is caught; registered as a required status check; runs on every PR |
| API-01 | PS-3 | Public types: operations, transactions, results, conflict results (E-03..E-06) | crates/cube-editing-api/src/types | listed types | mock-based contract tests | module | no behaviour in the crate |
| API-02 | PS-3 | Ports: IEditingService, IGeometryBackend + capabilities, ITransactionObserver (E-07, E-11, E-12; ACR-002) | crates/cube-editing-api/src/ports | listed ports | mock conformance | module | backends compile without Orchestration |
| API-03 | PS-3 | AuthorizationContext (E-02) with origin mapping to ActorKind | crates/cube-editing-api/src/auth.rs | AuthorizationContext | denial leaves no state | module | mapping documented; LocalClient origin gives no extra rights |
| API-04 | PS-3 | Storage and recovery ports: IDeltaStore, ISnapshotStore, IOperationJournal, IBaseProvider (supplies base generation to recovery), checkpoint/delta types (E-09, E-10) | crates/cube-editing-api/src/storage.rs | listed ports | mock tests | module | History and Persistence need no mutual dependency; recovery needs no WorldGen dependency |
| API-05 | PS-3 | Public-API snapshot test and contract-change tooling | xtask, crates/cube-editing-api/tests | CLI | snapshot diff | tooling | API diff fails CI without an accepted change request |

## 3. Core Agent
**Crate / area:** cube-core. **Acceptance authority:** Integration Owner (engineering); project owner (contracts). **Forbidden scope:** any dependency, I/O, threads (except sync primitives), clocks, randomness, engine types, serialization formats, persistence, network, WorldGen logic, history, tools.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| CORE-01 | PS-1 | Layer model: GeometryAsset, Layer, LayerKind, LayerData::{Voxel,Sdf,Mesh}, Representation tag (C-01 core slice, C-02, C-03) | crates/cube-core/src/layer/{mod,kind}.rs | Layer, LayerKind, LayerData | contract + property tests | module | LayerData is a closed enum; no voxel-only type in the public API |
| CORE-02 | PS-1 | CellKey{layer,level,coord}, Region, coordinate conversions (C-06, C-07 PROVISIONAL) | crates/cube-core/src/{coord,key,region}.rs | CellKey, Region, VoxelPos | negative-coordinate symmetry, level round trip | module | spike coordinate tests ported and green |
| CORE-03 | PS-1 | Channels, cell payload, cell state: ChannelSet, CellPayload::{Channels,MeshPage}, CellState (Present, KnownEmpty, NotLoaded, OutsideBounds, LoadFailure) (C-08, C-09 PROVISIONAL) | crates/cube-core/src/cell/{mod,channel,state}.rs | ChannelSet, CellPayload, CellState | storage property tests; spike brick tests ported | module | absent data is never conflated with empty data |
| CORE-04 | PS-1 | GeometryTransform: integer-only authoritative representation per D-T (C-04) | crates/cube-core/src/transform.rs | GeometryTransform | composition and determinism tests | module | no floating point in authoritative state (compile-time lint + test) |
| CORE-05 | PS-1 | RevisionToken (Asset, Layer, Cell scopes), epoch, RegionStamp (C-16) | crates/cube-core/src/revision.rs | RevisionToken, RegionStamp | ordering, incomparability, overflow | module | spike revision tests ported; overflow is a critical error, never wraps |
| CORE-06 | PS-1 | Commit: SharedAsset, immutable AssetState, commit gate, CommitBatch (atomic across layers of one asset), dirty regions (C-18, C-01) | crates/cube-core/src/{asset,commit,dirty}.rs | SharedAsset::{snapshot, commit}, CommitBatch | atomicity, isolation, loom gate tests | module | spike asset tests ported; loom green; all-or-nothing |
| CORE-07 | PS-1 | OperationId, ActorKind {User, System, Recovery}, error envelope (C-17, C-19, with the ACR-004 part A amendment to C-17) | crates/cube-core/src/{ids,error}.rs | OperationId, ActorKind, Error | code uniqueness, envelope fields | module | every error code has a negative test; formats versioned per D-R8 |
| CORE-08 | PS-1 | Mutation boundary: capability handles UserCommit, SystemCommit, RecoveryCommit; in-memory commit log; no public mutating API besides handles (ACR-004 part A) | crates/cube-core/src/boundary.rs | handle types, CommitLog (test-time) | compile-fail tests (trybuild), log completeness test | module | a revision increment without a commit-log entry is impossible (test) |
| CORE-09 | PS-1 | System-commit and recovery ports: Materialise, Evict, Restore (types + implementation behind the handles) (ACR-004 part A). The callers (Materialiser, persistence loader) are NOT part of PS-1 | crates/cube-core/src/system_commit.rs | SystemCommit, RecoveryCommit methods | atomic, idempotent, unreachable from a User handle | module | call-site audit passes; client-facing crates cannot name the handles |
| CORE-10 | PS-1 | CORE API FREEZE v0.1: publish the public-API snapshot of CORE-01..07 (the event CF-1) | crates/cube-core/api-snapshot.txt | API snapshot | API snapshot test | snapshot + contract tests | CI fails on any later public-API difference without an accepted change request |
| CORE-11 | PS-1 | Export/import views (not a file format) and stable content hash | crates/cube-core/src/{view,hash}.rs | AssetView, content_hash | roundtrip; golden hashes | module | no serialization format defined; hash algorithm and byte order fixed by the contract |
| CORE-15 | PS-2 | GeometryBounds data (C-05), separate approval | crates/cube-core/src/bounds.rs | GeometryBounds | bounds invariants | module | C-05 Owner-Approved before merge |
| CORE-16 | PS-2 | World identity value types WorldSeed and GeneratorVersion (plain newtypes, no behaviour; support for ACR-004 parts B and C, NOT part of PS-1) | crates/cube-core/src/world_ids.rs | WorldSeed, GeneratorVersion | newtype tests | module | additive to the frozen Core API; no WorldGen or Persistence logic in Core |
| CORE-12 | PS-5 | MaterialMapping palette data (C-10), separate approval and RG-1 | crates/cube-core/src/palette.rs | MaterialPalette | palette invariants | module | C-10 Owner-Approved after Voxel Architecture revalidation |
| CORE-14 | PS-8 | Composite data and junction data (C-13, C-14, C-15), after ACR-003 | crates/cube-core/src/{composite,junction}.rs | CompositePart, AssemblyGraph | graph invariants | module | ACR-003 decided; no operations in Core |
| CORE-13 | PS-9 | Resolution profile and refinement region data (C-11, C-12), separate approval and RG-1 | crates/cube-core/src/resolution.rs | ResolutionProfile, RefinementRegion | data invariants | module | C-11/C-12 Owner-Approved after revalidation |

## 4. Processing Agent
**Crate / area:** cube-processing. **Acceptance authority:** Integration Owner. **Forbidden scope:** editing, orchestration, backends, persistence, network, engine, async runtimes, any write to Core.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| PROC-01 | PS-2 | Generic snapshot with neighbour apron (PR-01) | crates/cube-processing/src/snapshot.rs | Snapshot, Apron | immutability under commits; apron correctness | module | spike snapshot tests ported |
| PROC-02 | PS-2 | Provider registry and dependency graph (PR-02, PR-07) | crates/cube-processing/src/{graph,provider}.rs | Provider, DerivedKind, DependencyGraph | invalidation by dirty regions; no cycles | module | derived stamp = input stamp set |
| PROC-03 | PS-2 | Executor, task state machine, priorities (PR-03, PR-06) | crates/cube-processing/src/{executor,task}.rs | Executor, Task, Priority | state machine; per-class budgets | module | no async runtime dependency |
| PROC-04 | PS-2 | Cancellation and deadlines (PR-05) | crates/cube-processing/src/cancel.rs | CancellationToken | cancel at each state | module | latency bound documented |
| PROC-05 | PS-2 | Stale rejection and DerivedSlot (PR-04) | crates/cube-processing/src/slot.rs | DerivedSlot<T> | race tests (loom); stale never published | module | spike stale tests ported |
| PROC-06 | PS-2 | Memory caps (OD-R4) and bounds/BVH slots (PR-08) | crates/cube-processing/src/{caps,bounds}.rs | PinBudget, bounds slots | cap enforcement; TTL expiry; synthetic mobile budget (no device) | module | caps enforced with a synthetic budget |

## 5. Agent A
**Crate / area:** cube-orchestration. **Acceptance authority:** Integration Owner. **Forbidden scope:** backend/tool/history/persistence implementations, network, a second commit path.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| ORCH-01 | PS-4 | Transaction lifecycle and rollback (E-04) | crates/cube-orchestration/src/txn.rs | OrchestrationService | rollback atomicity | module | no half-applied state |
| ORCH-02 | PS-4 | Backend registry and dispatch via capabilities (ACR-002) | crates/cube-orchestration/src/registry.rs | BackendRegistry | dispatch matrix with fakes | module | no backend import |
| ORCH-03 | PS-4 | Expected-token validation, conflict results, idempotent retry by OperationId | crates/cube-orchestration/src/validate.rs | none | conflict matrix; replay | module | retry never double-applies |
| ORCH-04 | PS-4 | Dirty propagation and observer notification | crates/cube-orchestration/src/dirty.rs | none | ordering tests | module | observers see post-commit state only |
| ORCH-05 | PS-4 | Authorisation and the single user commit path (UserCommit handle only) | crates/cube-orchestration/src/commit_path.rs | none | no-bypass negative tests | module | spike cube-editing behaviours reproduced through this path |

## 6. Agent B
**Crate / area:** cube-backend-voxel. **Acceptance authority:** Integration Owner. **Forbidden scope:** orchestration, tools, history, persistence, other backends, direct commit.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| VOX-01 | PS-5 | IGeometryBackend for Voxel: set/clear/fill/remove/copy/transform region | crates/cube-backend-voxel/src/ops.rs | backend impl | conformance + spike editing tests | module | write sets carry expected cell revisions |
| VOX-02 | PS-5 | Faces provider (reference oracle) from snapshot + apron [provider ownership pending O-1] | crates/cube-backend-voxel/src/faces.rs | provider | brute-force definition test | module | spike chunk_mesh tests ported |
| VOX-03 | PS-5 | Greedy provider, material-aware, SeamRule [pending O-1] | crates/cube-backend-voxel/src/{greedy,seams}.rs | provider | greedy=faces coverage, no T-junctions, determinism | module | spike greedy/tjunction tests ported (20) |
| VOX-04 | PS-5 | Section dirty rule and incremental rebuild | crates/cube-backend-voxel/src/sections.rs | dirty-section mapper | incremental equals full rebuild | module | spike sections tests ported (11) |
| VOX-05 | PS-5 | Collision and LOD providers for voxel [pending O-1] | crates/cube-backend-voxel/src/{collision,lod}.rs | providers | property + golden | module | budgets documented |

## 7. Agent C
**Crate / area:** cube-backend-sdf. **Acceptance authority:** Integration Owner. **Forbidden scope:** as Agent B.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| SDF-01 | PS-5 | SDF backend ops: sampling, density modification, quantised field | crates/cube-backend-sdf/src/ops.rs | backend impl | conformance; convention (negative inside) | module | surface preserved by ops |
| SDF-02 | PS-5 | Surface Nets provider with apron [pending O-1] | crates/cube-backend-sdf/src/surface_nets.rs | provider | watertightness, determinism, seams | module | no cracks at cell joints |
| SDF-03 | PS-5 | Gradient/normal and collision provider [pending O-1] | crates/cube-backend-sdf/src/{gradient,collision}.rs | providers | numeric tolerance | module | tolerance documented |

## 8. Agent D
**Crate / area:** cube-backend-mesh. **Acceptance authority:** Integration Owner. **Forbidden scope:** as Agent B.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| MSH-01 | PS-5 | Mesh backend ops on mesh pages | crates/cube-backend-mesh/src/ops.rs | backend impl | conformance; topology invariants | module | mesh revision tracked per page |
| MSH-02 | PS-5 | Mesh validation and normal recalculation | crates/cube-backend-mesh/src/validate.rs | validator | manifold/degenerate | module | typed errors |
| MSH-03 | PS-5 | Mesh collision provider [pending O-1] | crates/cube-backend-mesh/src/collision.rs | provider | golden | module | budgets documented |

## 9. Agent E
**Crate / area:** cube-tool-sculpt. **Acceptance authority:** Integration Owner. **Forbidden scope:** direct commit, backend implementations, history, persistence.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| SCU-01 | PS-6 | Brush model, falloff, parameters | crates/cube-tool-sculpt/src/brush.rs | Brush | determinism | module | pure functions |
| SCU-02 | PS-6 | Sculpt operations per backend capability | crates/cube-tool-sculpt/src/tools.rs | tool ops | transaction integration; stale snapshot rejected | module | async revision safe |
| SCU-03 | PS-6 | Backend rules and explicit conversion requests | crates/cube-tool-sculpt/src/convert.rs | none | conversion tests | module | no implicit conversion |

## 10. Agent F
**Crate / area:** cube-tool-boolean. **Acceptance authority:** Integration Owner. **Forbidden scope:** as Agent E.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| BOO-01 | PS-6 | BooleanPolicy contract (B-7) and policy implementation | crates/cube-tool-boolean/src/policy.rs | BooleanPolicy | policy tests | module | contract written and Owner-Approved first |
| BOO-02 | PS-6 | Per-representation boolean via capabilities | crates/cube-tool-boolean/src/ops.rs | tool ops | property tests per representation | module | results valid |
| BOO-03 | PS-6 | Explicit conversion and async safety | crates/cube-tool-boolean/src/convert.rs | none | stale rejection | module | no hidden conversion |

## 11. Agent G
**Crate / area:** cube-history. **Acceptance authority:** Integration Owner. **Forbidden scope:** persistence implementation, orchestration implementation, direct commit, system-commit port.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| HIST-01 | PS-7 | Observer, deltas, command stacks per actor | crates/cube-history/src/{stack,delta}.rs | HistoryService | restoration tests | module | System and Recovery commits are excluded |
| HIST-02 | PS-7 | Undo/redo through IEditingService with expected post tokens (OD-R5) | crates/cube-history/src/undo.rs | undo/redo | divergence conflict; property tests | module | undo is a forward commit |
| HIST-03 | PS-7 | Checkpoints and memory limits | crates/cube-history/src/checkpoint.rs | none | limit behaviour | module | bounded memory |
| HIST-04 | PS-7 | History scope abstraction (per-actor stacks, local vs server scope) without networking | crates/cube-history/src/scope.rs | none | per-actor isolation | module | no cross-actor undo without permission |

## 12. Composite Agent
**Crate / area:** cube-composite. **Acceptance authority:** Integration Owner. **Forbidden scope:** owning the data model (in Core), merging parts automatically.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| COMP-01 | PS-8 | Attach/detach through transactions | crates/cube-composite/src/attach.rs | ops | graph invariants | module | no cycles |
| COMP-02 | PS-8 | Constraints | crates/cube-composite/src/constraints.rs | none | solver determinism | module | typed violations |
| COMP-03 | PS-8 | Transform propagation | crates/cube-composite/src/propagate.rs | none | determinism | module | order independent |

## 13. Resolution Agent
**Crate / area:** cube-resolution. **Acceptance authority:** Integration Owner. **Forbidden scope:** direct commit; Core data changes (requests go to the Core Agent).

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| RES-01 | PS-9 | Resampling between levels | crates/cube-resolution/src/resample.rs | ops | round trips | module | error bounds documented |
| RES-02 | PS-9 | Refinement regions | crates/cube-resolution/src/refine.rs | ops | mixed-resolution tests | module | undoable |
| RES-03 | PS-9 | Transitions and seam handling with providers | crates/cube-resolution/src/transition.rs | none | seam tests | module | no cracks between levels |

## 14. Agent H
**Crate / area:** cube-persistence. **Acceptance authority:** Integration Owner; project owner (formats). **Forbidden scope:** history, orchestration, backends, worldgen, defining formats before owner approval, publishing without directory fsync.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| SPK-PERS | PS-0 | Persistence durability spike (fsync, directory fsync, atomic rename, crash points) on a spike branch; NO format approval | spike branch only | none | crash experiments | spike report | report states results per platform; never merged |
| PERS-01 | PS-10 | Cell/asset codec with schema versions, checksums, versioned OperationId and revision fields (P-02) | crates/cube-persistence/src/codec.rs | codec | roundtrip; corruption detection | module | every record carries a schema version; no format is Owner-Approved yet |
| PERS-02 | PS-10 | Manifest with atomic publish and directory fsync semantics (P-01, P-05) | crates/cube-persistence/src/manifest.rs | manifest API | atomicity with FaultFs | module | old or new manifest, never mixed; directory fsync verified |
| PERS-03 | PS-10 | Snapshot store (ISnapshotStore, P-04) recording base generator version and revision boundary | crates/cube-persistence/src/snapshot.rs | store | verify step | module | snapshot invalid until verified |
| PERS-04 | PS-10 | Mutation journal and delta store (IDeltaStore, IOperationJournal, P-03) with versioned OperationId, ActorKind and revision records | crates/cube-persistence/src/journal.rs | store | truncated tail; checksums; gapless sequence | module | survives sudden termination; replay yields identical commit sequence |
| PERS-05 | PS-10 | Compaction protocol (AOD-21): write, fsync, verify, atomic manifest, fsync, truncate | crates/cube-persistence/src/compact.rs | none | crash at every step; idempotence | module | prefix truncated only after verification |
| PERS-06 | PS-10 | Recovery of the user layer: base from IBaseProvider, then snapshot, then journal tail via RecoveryCommit; hash, epoch, revision, generator-version checks | crates/cube-persistence/src/recovery.rs | recover() | fallbacks; mismatch refusal; idempotent | module | Recovery actor only through the port; no WorldGen dependency |
| PERS-07 | PS-10 | Migration framework (P-07); generator-version policy hooks (pin, bake, explicit migrate) | crates/cube-persistence/src/migrate.rs | none | rehearsal | module | never silent |
| PERS-08 | PS-10 | World record: WorldSeed, GeneratorVersion, config, manifest reference | crates/cube-persistence/src/world.rs | WorldRecord | roundtrip | module | untouched world = record only |
| PERS-09 | PS-10 | Composite structure persistence (P-08), using only the Core composite data | crates/cube-persistence/src/composite.rs | none | roundtrip | module | no dependency on cube-composite |

## 15. UNASSIGNED (WorldGen) - owner decision
**Crate / area:** cube-worldgen. **Acceptance authority:** Integration Owner. **Forbidden scope:** direct commit; floats in world math; clocks; global state; orchestration; persistence.
**R-UNOWNED:** these tasks cannot start until the project owner names the owner (GOV-03).

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| LEG-01 | PS-11 | Remove WorldStreamer direct commits, set_voxel user path and world-state-v0 (ACR-004 E, part 1) | removal PR | none | regression | PR | no direct commit remains in cube-worldgen (archcheck) |
| WGEN-01 | PS-11 | 11A Pure generator + version registry (read-only Core API only) | crates/cube-worldgen/src/generator | generate() | spike determinism tests ported | module | pinned hashes per version; no floats in world math |
| WGEN-02 | PS-11 | 11B Service: queue, cancellation, cache, chunk states | crates/cube-worldgen/src/service.rs | WorldGenService | spike world.rs tests as specification | module | no stale result applied |
| WGEN-03 | PS-11 | 11B Materialiser through SystemCommit (ACR-004 B) | crates/cube-worldgen/src/materialiser.rs | none | idempotent materialise/evict; archcheck | module | only writer of generated data |
| WGEN-04 | PS-11 | 11B Interest handling and eviction (user data never evicted before it is durable) | crates/cube-worldgen/src/interest.rs | none | hysteresis; user data kept | module | server-driven only |
| WGEN-05 | PS-11 | Cross-device hash harness (software side) | crates/cube-worldgen/tests/xdevice.rs | none | harness | tests | device execution is DG-2 |

## 16. UNASSIGNED (Client Runtime) - owner decision
**Crate / area:** cube-client. **Acceptance authority:** Integration Owner. **Forbidden scope:** godot, authority over world state.
**R-UNOWNED:** these tasks cannot start until the project owner names the owner (GOV-03).

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| CLI-00 | PS-12 | Client skeleton with a mock transport (parallel preparation only) | crates/cube-client | ClientWorld skeleton | unit | skeleton | not production; no authority |
| CLI-01 | PS-12 | Chunk cache and interest view | crates/cube-client/src/cache.rs | ClientWorld | cache consistency | module | read-only cache |
| CLI-02 | PS-12 | Mesh scheduler: closest-first, per-frame budget, stale drop | crates/cube-client/src/mesh_sched.rs | MeshScheduler | budget and stale tests | module | main-thread cost within the budget on the desktop profile |
| CLI-03 | PS-12 | LOD selection and view-radius profiles per device class | crates/cube-client/src/lod.rs | none | profile tests | module | memory bounds |
| CLI-04 | PS-12 | Input intents and prediction scaffolding | crates/cube-client/src/intent.rs | Intent | deterministic replay | module | no authority |
| CLI-05 | PS-13 | Client protocol integration and prediction/reconciliation | crates/cube-client/src/net.rs | none | reconciliation properties | module | client never authoritative |

## 17. UNASSIGNED (Engine Adapter, O-5) - owner decision
**Crate / area:** cube-godot-adapter, godot-client. **Acceptance authority:** Integration Owner. **Forbidden scope:** algorithms, undo, persistence, world logic, OS.execute, panics across FFI.
**R-UNOWNED:** these tasks cannot start until the project owner names the owner (GOV-03).

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| ADP-01 | PS-12 | Shrink the adapter to translation (values, views, buffers) | crates/cube-godot-adapter/src | GDExtension classes | API snapshot; desktop smoke | module | no algorithm left in the adapter |
| ADP-02 | PS-12 | GDScript = UI and glue only; remove test code from the client | godot-client/*.gd | none | no_child_processes | scripts | AOD-07 and AOD-12 hold |
| ADP-03 | PS-12 | Error wrapping, catch_unwind, panic policy test (including panic=abort decision) | crates/cube-godot-adapter | none | panic injection | module | no abort across the FFI |
| LEG-02 | PS-12 | Remove WorldBridge edit_voxel, fill_box, undo/redo and apply_batch (ACR-004 E, part 2) | removal PR | none | regression | PR | adapters contain no history and no write path |

## 18. UNASSIGNED (Networking Adapter, O-3) - owner decision
**Crate / area:** cube-net. **Acceptance authority:** Integration Owner. **Forbidden scope:** server logic, persistence, engines.
**R-UNOWNED:** these tasks cannot start until the project owner names the owner (GOV-03).

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| SPK-NET | PS-0 | Transport spike (TCP, QUIC/UDP, library, control/data split) on a spike branch; NO transport approval | spike branch only | none | loss/jitter/suspend experiments | spike report | report meets the criteria of the matrix OAM-05; never merged |
| NET-01 | PS-13 | Transport contract: delivery classes, ordering, session semantics (trait + document) | crates/cube-net/src/transport.rs | Transport trait | contract tests | module | no carrier named in the contract |
| NET-02 | PS-13 | Protocol and session types incl. handshake and authentication MESSAGE types, versioning (I/O-free) | crates/cube-net/src/protocol | messages | conformance; fuzz | module | testable without sockets |
| NET-03 | PS-13 | In-process transport | crates/cube-net/src/inproc.rs | transport | shared transport suite | module | same semantics as the external transport |
| NET-04 | PS-13 | External transport implementation | crates/cube-net/src/external.rs | transport | shared suite; loss/jitter | module | mobile network tests belong to DG-3 |

## 19. UNASSIGNED (Server Runtime) - owner decision
**Crate / area:** cube-server. **Acceptance authority:** Integration Owner. **Forbidden scope:** godot, UI, process calls, a second authority path, client-supplied file paths.
**R-UNOWNED:** these tasks cannot start until the project owner names the owner (GOV-03).

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| SRV-01 | PS-13 | Server runtime: lifecycle and the EMBEDDED host over the in-process transport; shared runtime for both hosts; recovery wiring (IBaseProvider from WorldGen, PERS-06) | crates/cube-server/src | ServerRuntime | lifecycle suite on both hosts | module | AOD-22 rules; one runtime, two hosts |
| SRV-02 | PS-13 | Server session management and authentication INTEGRATION | crates/cube-server/src/session.rs | none | authority tests | module | no unauthenticated edit path |
| SRV-03 | PS-13 | Tick loop and single writer | crates/cube-server/src/tick.rs | none | determinism; soak | module | no concurrent commit paths |
| SRV-04 | PS-13 | Interest management and replication | crates/cube-server/src/replicate.rs | none | consistency under loss | module | client cache equals server state |
| SRV-05 | PS-13 | Reconnect, catch-up, idempotent retry | crates/cube-server/src/reconnect.rs | none | replay tests | module | reconnect within the budget (device figure in DG-3) |
| SRV-06 | PS-13 | Permissions and protected regions | crates/cube-server/src/perm.rs | none | permission matrix | module | denial leaves no state |
| SRV-07 | PS-13 | Server-side history integration (per-actor stacks over the session) | crates/cube-server/src/history.rs | none | per-actor isolation | module | no cross-actor undo without permission |
| SRV-08 | PS-13 | DEDICATED host over the external transport (same runtime as SRV-01) | crates/cube-server/src/host/dedicated.rs | none | lifecycle and conformance suites on the dedicated host | module | differs from the embedded host only in launch, lifecycle and transport (AOD-22) |

## 20. Agent I (QA and Benchmarks)
**Crate / area:** cube-testkit, cube-integration-tests, CI, device tooling. **Acceptance authority:** Integration Owner. **Forbidden scope:** production code in other crates; changing acceptance thresholds.

| ID | Stage | Scope | Files/modules | Public API | Required tests | Deliverables | Acceptance |
|---|---|---|---|---|---|---|---|
| GOV-06 | PS-0 | CI IMPLEMENTATION: workspace skeleton PR with pinned toolchain and running fmt, clippy, test, deny, archcheck v0 | .github/workflows/*, rust-toolchain.toml, deny.toml, Cargo.toml | none | CI self-run on a seeded failing PR | running CI | a workflow run URL shows every required check green on an empty skeleton and red on seeded failures; branch protection on master lists them as required status checks (API output attached) |
| TEST-01 | PS-0 | cube-testkit skeleton: deterministic strategies, fakes | crates/cube-testkit | test helpers | self-tests | crate | used by at least two crates |
| TEST-02 | PS-1 | Golden-hash harness (software, any host); device runs belong to DG-2 | crates/cube-testkit | golden runner | harness self-tests | harness | hashes reproduce on the CI host |
| TEST-05 | PS-1 | Core property and loom suites: commit atomicity, snapshot isolation, monotonic revisions | crates/cube-integration-tests/tests/core_properties.rs | none | proptest, loom | suites | suites run in CI and fail on seeded violations |
| TEST-06 | PS-1 | Mutation-boundary negative tests available in PS-1: compile-fail (trybuild) tests for handles, commit-log completeness test, archcheck seeded violations | crates/cube-integration-tests/tests/mutation_boundary.rs | none | compile-fail + archcheck | tests | every seeded bypass fails CI |
| TEST-08 | PS-2 | Processing concurrency and cancellation suites (loom, stress) | crates/cube-integration-tests/tests/processing_concurrency.rs | none | loom, stress | suites | no stale publication under any interleaving explored |
| TEST-03a | PS-4 | Backend conformance suite (independent of any renderer choice) | crates/cube-integration-tests/src/conformance/backend.rs | suite | self-check against fakes | suite | each backend can run it |
| TEST-10 | PS-4 | Negative tests through the real Orchestration path: a user request cannot yield a System or Recovery commit; clients and adapters cannot commit | crates/cube-integration-tests/tests/mutation_negative.rs | none | compile-fail + archcheck | tests | every seeded bypass fails CI |
| ITG-01 | PS-10 | Integration test: journal produced by user commits replays to the identical state (History not required) | integration tests (composition crate) | none | replay equality | test | replay equals live state for the spike scenarios |
| QA-P2 | PS-10 | Persistence crash-recovery matrix on desktop (all numbered crash points, truncated tail, corrupted files, version mismatch, double recovery) | crates/cube-integration-tests/tests/persistence_crash.rs | none | fault injection | suite | all crash points recover; no unverified snapshot ever becomes sole truth |
| TEST-04 | PS-10 | Fault-injecting filesystem and crash-point runner | crates/cube-testkit/src/faultfs | FaultFs | self-tests | harness | can crash after any numbered I/O step |
| ITG-02 | PS-11 | Integration test: generation, user journal and recovery produce the identical world | integration tests | none | end-to-end | test | replay of the spike scenarios equals live state |
| TEST-09 | PS-11 | Determinism suite on the CI host (pinned hashes, order independence) | crates/cube-integration-tests/tests/determinism.rs | none | golden | suite | device equality is DG-2, not this task |
| RND-01 | PS-12 | Render regression tests (pixel identity faces/greedy, T-junction) on the desktop host | crates/cube-integration-tests/tests/render_regression.rs | none | image tests | tests | spike pixel checks reproduced |
| TEST-03b | PS-13 | Transport conformance suite, run on in-process and external transports and on both hosts | crates/cube-integration-tests/src/conformance/transport.rs | suite | self-check | suite | a transport failing the suite cannot ship |
| TEST-07 | PS-13 | Authority and security tests (client cannot commit or call system ports; no client-supplied paths; replay rejected; protocol fuzzing) | crates/cube-integration-tests/tests/authority_security.rs | none | security tests | suite | all negative cases fail closed |
| DG-1 | PS-14 | DEVICE GATE 1 - renderer validation: faces vs greedy on physical devices | reports | none | device runs | report | input to the owner decision GOV-13; not a stage exit of PS-5 |
| DG-2 | PS-14 | DEVICE GATE 2 - mobile determinism and durability (hashes, fsync behaviour) on physical devices | reports | none | device runs | report | hash equality or recorded owner deferral |
| DG-3 | PS-14 | DEVICE GATE 3 - mobile release: memory, cold start, thermal, battery, 4 KB/16 KB pages, signing, network behaviour | reports | none | device matrix | report | every matrix device passes |
| QA-02 | PS-14 | Benchmarks with stored baselines (desktop CI) and device runner scripts | crates/cube-integration-tests/benches/, scripts/ | none | criterion | benches | regression gate on desktop; device thresholds set from DG data |
| QA-04 | PS-14 | Full regression, stress and soak on desktop | CI nightly | none | soak | report | no growth or crashes in the soak window |
| QA-06 | PS-14 | Device validation tooling (scripts moved from the spike), iOS pipeline | scripts/ | none | script tests | tooling | scripts are reproducible and refuse to run without a physical device |


## Task counts

| Owner | Tasks |
|---|---|
| Project owner | 22 |
| Main Agent / Integration Owner | 10 |
| Agent I (QA and Benchmarks) | 22 |
| Agent H | 10 |
| UNASSIGNED (Networking Adapter, O-3) - owner decision | 5 |
| Core Agent | 16 |
| Processing Agent | 6 |
| Agent A | 5 |
| Agent B | 5 |
| Agent C | 3 |
| Agent D | 3 |
| Agent E | 3 |
| Agent F | 3 |
| Agent G | 4 |
| Composite Agent | 3 |
| Resolution Agent | 3 |
| UNASSIGNED (WorldGen) - owner decision | 6 |
| UNASSIGNED (Client Runtime) - owner decision | 6 |
| UNASSIGNED (Engine Adapter, O-5) - owner decision | 4 |
| UNASSIGNED (Server Runtime) - owner decision | 8 |
| **Total** | **147** |
