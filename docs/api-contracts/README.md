# Cube — API Contracts Index (Geometry)

**Status:** DRAFT 0.1. **No contract in this directory is approved.** Approval is recorded by the Integration Owner in `docs/decisions/`.
**Implementation status (INT-01, 2026-11-09):** the Core slice of the P0-alpha contracts is **Implemented** in `crates/cube-core` (merged into `dev`, required CI checks green, contract tests in the crate); PC-1 contracts are **Implemented under Provisional approval**; **no contract is Frozen** (approval axis unchanged; see registry DEC-071 and DEC-076).

| Contract | Implemented slice | Not implemented (open) |
|---|---|---|
| C-01 GeometryAsset | immutable asset of layers, snapshots, commit gate | junctions, composite references, metadata schema |
| C-02 GeometryLayer | id, representation, transform, data | resolution profile, material mapping, modifier stack, collision profile |
| C-03 GeometryLayerType | closed `LayerKind` {Voxel, Sdf, Mesh} | extension type registry (a new representation needs an ACR) |
| C-04 GeometryTransform | integer translation, 24 rotations, power-of-two scale, exact compose/apply | negative scale in compose/apply (floor semantics), shear |
| C-06 Region | half-open box at a level, intersection, brick span and iteration | non-box shapes (tool level) |
| C-16 RevisionToken | tokens for asset, layer, cell, region, derived; epoch; never-wrapping `u64`; versioned `RegionStamp` | derived-slot publication protocol (Processing) |
| C-17 OperationId | `(scope, sequence)`, `ActorKind` with stable codes and reserved `Simulation` | dedupe window (server runtime) |
| C-18 DirtyRegion | brick-granular dirty regions with actor and operation | sub-brick granularity |
| C-19 Result/Error | stable codes, severity, category, module; one negative test per code | `OperationId` in the error context (attached by the Editing layer) |
| C-07 BrickKey (PC-1) | `CellKey(layer, level: i8, brick [i64; 3])`, total order | none known |
| C-08 BrickStorage (PC-1) | sparse store, five distinct cell states, bounds, copy-on-write payloads, export/import views | `AcquirePin`, per-brick export (views are provided instead) |
| C-09 GeometryChannel (PC-1) | Material, Density, Attribute channels as raw `u16` with Constant/Dense encodings | attribute registry, density quantisation scale |

**Authority:** Architecture Update v1.0 (AU-1.0) > Geometry Editing Specification > Agent Development Specification > these contracts (Agent Spec §2).
**Rule:** contracts are conceptual. They contain **no final language-specific signatures** (language and engine are open).

## 1. Files

| File | Owner (module) | Layer |
|---|---|---|
| `core-contracts.md` | Core Agent | 0 |
| `processing-contracts.md` | Processing Agent | 1 |
| `editing-contracts.md` | Main Agent / Integration Owner | 2 |
| `persistence-contracts.md` | Agent H | 8 |
| `revision-and-snapshot-semantics.md` | Integration Owner (semantics) | cross-cutting |

### 1.1 Draft additions of 2026-11-05 (outside the 45 + 3 count, no tier assigned)

| File | Contracts | Status |
|---|---|---|
| `worldgen-contracts.md` | WG-01…WG-10 | Draft 0.1.0-draft, Proposed |
| `system-commit-contracts.md` | SC-01…SC-10 (proposes extensions of C-16/C-17/C-18; see ACR-004, Pending) | Draft, Proposed |
| `world-persistence-contracts.md` | WP-01…WP-10 (reconciliation draft for OI-4; P-01…P-07 untouched) | Draft, Proposed |
| `server-runtime-contracts.md` | SR-01…SR-10 | Draft, Proposed |

None is approved; the matrix counts, tiers and P0/P1 below are unchanged.

## 2. Common Conventions (apply unless a contract says otherwise)

1. **Threading:** value types are immutable and thread-safe. Authoritative data is mutated only inside the commit primitive; readers use snapshots or read views.
2. **Errors:** `Result<T> | Error`; `Error = {StableCode, Module, Context, OperationId?, RecoveryPolicy, Severity, Category}`; no memory references in errors; exceptions only for unexpected critical failures (AU-1.0 §14).
3. **Serialization:** every persisted/networked structure carries a schema version. Unknown optional fields are preserved; unknown required features are a controlled error.
4. **Compatibility:** additive optional changes are minor; anything else is breaking and needs an Architecture Change Request (ACR).
5. **Revision & Dirty:** every mutating contract states which revision scope it increments and which ChangeType it emits.
6. **Tests:** every contract needs contract tests (API shape/semantics), property tests where invariants exist, and negative tests for each error.
7. **Ownership/lifetime:** no raw internal references cross module boundaries; use handles, views, or pins.
8. **Status vocabulary:** `Draft` → `Proposed` → `Approved` → `Implemented` → `Frozen`. All contracts below are `Draft`.

## 3. API Contract Matrix

### 3.1 Approval Tiers (single definition used by all documents)

Tiers are **approval gates** tied to the build stages in the Integration & QA Pack section 5. A tier must be Approved before the stage it gates may start.

| Tier | Contracts | Count | Gates |
|---|---|---|---|
| P0 | C-01…C-19 (all Geometry Core contracts incl. RevisionToken, OperationId, DirtyRegion, Result/Error) | 19 | Stage 1 (Geometry Core) |
| P1 | PR-01…PR-08 (Processing); E-02…E-07, E-09, E-11, E-12 (Editing API, backend port, observer port, delta) | 17 | Stage 2 (Processing), Stages 3-5 (Editing API, Orchestration, Backends). P1 may be reviewed in parallel with Stage 1 |
| P2 | E-08, E-10, `IDeltaStore`, `ISnapshotStore`, P-01…P-07 | 9 numbered + 2 ports | Stages 6-10 for the affected module (Tools/modifiers, History, Persistence) |
| P3 | PR-09, E-13, P-09 (interface boundaries); tool-level contracts not yet written | 3 + 5 | Stages 11-12 and each tool module's own start |

19 + 17 + 9 + 3 = 48 matrix rows (45 contracts + 3 boundaries). Rationale: Core cannot start without its own data contracts; Processing consumes Core and defines its own; Orchestration and backends cannot start without the transaction, operation, result, conflict, backend, delta and observer contracts; History and Persistence need checkpoints and storage ports only later.

### 3.2 Contract list

**Counting note (official wording):** the set is **45 contracts + 3 interface boundaries** (PR-09, E-13, P-09; boundary-only, no version yet), plus 2 unnumbered L2 storage ports (`IDeltaStore`, `ISnapshotStore`). The phrase "48 contracts" is not used: it counted the boundaries as contracts. Section ids E-01 (pointer to C-17) and P-08 (composite persistence note) are not contracts and carry no row. Plus two storage ports (`IDeltaStore`, `ISnapshotStore`) listed unnumbered. **Owner column = the single contract owner**; the agent that implements an operation or algorithm over the data is named in the consumers column.

Version scheme: `major.minor.patch[-draft]`. All contracts are `0.1.0-draft`; `1.0.0` is assigned on approval. Per-contract ownership, lifetime, access, thread-safety, revision semantics, errors and contract tests are in the file named in the *Spec file* column.

Legend: **Draft** = specified in this pass, awaiting approval. **Draft+OD** = specified, with named Open Decisions that block freezing. **Boundary** = only the interface boundary is specified.

| ID | Contract | Provider (owner) | Primary consumers | Spec file | State | Version | Tier | Blocking open decisions |
|---|---|---|---|---|---|---|---|---|
| C-01 | GeometryAsset | Core | all | core | Draft | 0.1.0-draft | P0 | metadata schema |
| C-02 | GeometryLayer | Core | all | core | Draft+OD | 0.1.0-draft | P0 | collision profile schema |
| C-03 | GeometryLayerType | Core | API, backends, tools | core | Draft+OD | 0.1.0-draft | P0 | extension registration |
| C-04 | GeometryTransform | Core | all | core | Draft+OD | 0.1.0-draft | P0 | f32/f64, epsilon |
| C-05 | GeometryBounds | Core | Processing (derived bounds, PR-08), backends | core | **Owner-Approved (minimal, DEC-079); Implemented (CORE-15)** | 0.1.0 | P0 | OBB/sphere not included |
| C-06 | GeometryRegion | Core | all | core | Draft+OD | 0.1.0-draft | P0 | non-AABB shapes |
| C-07 | BrickKey | Core | Voxel, Persistence, all | core | Draft+OD | 0.1.0-draft | P0 | int width, refinement addressing |
| C-08 | BrickStorage | Core | Processing, backends, Persistence | core | Draft+OD | 0.1.0-draft | P0 | layout, index order, compression |
| C-09 | GeometryChannel | Core | backends, tools | core | Draft+OD | 0.1.0-draft | P0 | attribute registry, widths |
| C-10 | MaterialMapping | Core | backends, Boolean, Sculpt | core | Draft+OD | 0.1.0-draft | P0 | global library interface |
| C-11 | ResolutionProfile | Core | backends, Resolution | core | Draft | 0.1.0-draft | P0 | value sets |
| C-12 | RefinementRegion | Core | backends, Resolution Agent (operations) | core | Draft+OD | 0.1.0-draft | P0 | transition algorithm |
| C-13 | Junction | Core | Boolean, Processing | core | Draft | 0.1.0-draft | P0 | blend params |
| C-14 | CompositePart | Core | Composite Agent (operations), Persistence | core | Draft+OD | 0.1.0-draft | P0 | ACR-003 |
| C-15 | AssemblyGraph | Core | Composite Agent (operations), Persistence | core | Draft+OD | 0.1.0-draft | P0 | ACR-003, constraint scope |
| C-16 | RevisionToken | Core | all | core + semantics | Draft+OD | 0.1.0-draft | P0 | OD-R1…R8 |
| C-17 | OperationId | Core | all | core | Draft+OD | 0.1.0-draft | P0 | OD-R8 |
| C-18 | DirtyRegion | Core | Processing, Orchestration, backends | core | Draft+OD | 0.1.0-draft | P0 | sub-brick granularity |
| C-19 | Result/Error envelope | Core | all | core | Draft | 0.1.0-draft | P0 | code scheme |
| PR-01 | Processing Snapshot | Processing | Orchestration, backends, Boolean, Persistence | processing | Draft+OD | 0.1.0-draft | P1 | pin vs copy, caps |
| PR-02 | Dependency Revision | Processing | Processing, Orchestration | processing | Draft+OD | 0.1.0-draft | P1 | granularity |
| PR-03 | Processing Task / AsyncJobHandle | Processing | Orchestration, Tools, Persistence | processing | Draft+OD | 0.1.0-draft | P1 | task kind extension |
| PR-04 | Processing Result | Processing | Orchestration | processing | Draft+OD | 0.1.0-draft | P1 | payload typing |
| PR-05 | CancellationToken | Processing | all async | processing | Draft | 0.1.0-draft | P1 | deadlines |
| PR-06 | Job Priority | Processing | submitters | processing | Draft+OD | 0.1.0-draft | P1 | weights |
| PR-07 | Dependency Graph registration | Processing | derived-data owners | processing | Draft+OD | 0.1.0-draft | P1 | provider ownership (O-1) |
| PR-08 | Spatial Hierarchy / Bounds / BVH update | Processing | queries, collision | processing | Draft+OD | 0.1.0-draft | P1 | query API home |
| PR-09 | Engine ports | Processing | Engine adapters | processing | Boundary | — | P3 | exact set |
| E-02 | AuthorizationContext | Editing API (input from session layer) | Orchestration, Networking | editing | **Owner-Approved (provisional parts, DEC-082); Implemented** | 0.1.0-draft | P1 | permission model |
| E-03 | EditingOperation | Editing API | Orchestration, Tools, History | editing | **Owner-Approved (provisional parts, DEC-082); Implemented** | 0.1.0-draft | P1 | Kind registry |
| E-04 | EditingTransaction | Editing API | Orchestration (implements), Tools, backends, History | editing | **Owner-Approved (provisional parts, DEC-082); Implemented** | 0.1.0-draft | P1 | nested/group, async |
| E-05 | EditingResult | Editing API | all | editing | **Owner-Approved (provisional parts, DEC-082); Implemented** | 0.1.0-draft | P1 | — |
| E-06 | ConflictResult | Editing API | Orchestration, Networking | editing | **Owner-Approved (provisional parts, DEC-082); Implemented** | 0.1.0-draft | P1 | merge policy |
| E-07 | IGeometryBackend / Capabilities | Editing API | Orchestration, backends, Tools | editing | **Owner-Approved (provisional parts, DEC-082); Implemented** | 0.1.0-draft | P1 | capability vocabulary |
| E-08 | ModifierStack | Editing API | Tools, evaluation | editing | Draft+OD | 0.1.0-draft | P2 | evaluation owner (O-2) |
| E-09 | GeometryDelta | Editing API | History, Persistence | editing | **Owner-Approved (provisional parts, DEC-082); Implemented** | 0.1.0-draft | P1 | delta encoding |
| E-10 | Checkpoint | Editing API | History, Persistence | editing | **Owner-Approved (provisional parts, DEC-082); Implemented** | 0.1.0-draft | P2 | frequency policy |
| E-11 | IEditingService | Editing API | Tools, Composite, History, Networking | editing | **Owner-Approved (provisional parts, DEC-082); Implemented** | 0.1.0-draft | P1 | signatures |
| E-12 | ITransactionObserver | Editing API | Orchestration (publish), History (sink) | editing | **Owner-Approved (provisional parts, DEC-082); Implemented** | 0.1.0-draft | P1 | durability ordering |
| E-13 | Networking boundary | Editing API | Networking Adapter | editing | Boundary | — | P3 | owner, protocol (O-3) |
| — | IDeltaStore / ISnapshotStore | Editing API | History, Persistence | editing §0 | Draft | 0.1.0-draft | P2 | — |
| P-01 | AssetManifest | Persistence | Persistence, tools | persistence | Draft+OD | 0.1.0-draft | P2 | container, format |
| P-02 | BrickSerialization | Persistence | Persistence | persistence | Draft+OD | 0.1.0-draft | P2 | compression |
| P-03 | DeltaSerialization | Persistence | History (via port) | persistence | Draft+OD | 0.1.0-draft | P2 | flush policy |
| P-04 | SnapshotSerialization | Persistence | History (via port) | persistence | Draft+OD | 0.1.0-draft | P2 | incremental |
| P-05 | StorageTransaction | Persistence | Persistence | persistence | Draft+OD | 0.1.0-draft | P2 | platform atomic publish |
| P-06 | IntegrityReport | Persistence | Core validation, QA | persistence | Draft | 0.1.0-draft | P2 | repair automation |
| P-07 | MigrationDescriptor | Persistence | Persistence | persistence | Draft | 0.1.0-draft | P2 | execution mode |
| P-09 | Import/Export adapters | unassigned | — | persistence | Boundary | — | P3 | owner (O-4) |
| — | BooleanPolicy | Agent F | Orchestration, backends | *not yet written* | **Missing** | — | P3 | next pass |
| — | BrushParameters / Sculpt tool interface | Agent E | Orchestration, backends | *not yet written* | **Missing** | — | P3 | next pass |
| — | Resampling/Refinement operation contract | Resolution Agent | backends | *not yet written* | **Missing** | — | P3 | next pass |
| — | Composite operation contract | Composite Agent | Orchestration | *not yet written* | **Missing** | — | P3 | next pass |
| — | History interface (undo/redo/limits) | Agent G | UI, Networking | *not yet written* | **Missing** | — | P3 | next pass |

Tool-level contracts (last five rows) were outside the minimum list requested for this pass and belong to their module owners after the tier that gates their stage is approved (see section 3).

## 4. Mapping of former QA Pack §2 names

| Former name | Now |
|---|---|
| GeometryAssetHandle / GeometryLayerHandle | handles of C-01 / C-02 |
| GeometryType | C-03 GeometryLayerType |
| BrickAddress | C-07 BrickKey |
| AttributeChannel | C-09 |
| MaterialPalette | C-10 |
| RevisionToken (Core/Processing) | C-16, single owner Core |
| DirtyRegion (Core/Processing) | C-18, single owner Core (ACR-001) |
| GeometrySnapshot | PR-01 |
| EditingTransaction / EditingCommand | E-04 / E-03 |
| GeometryOperationResult / EditingError | E-05 / C-19 |
| AsyncJobHandle / CancellationToken | PR-03 / PR-05 |
| ModifierDescriptor | E-08 |
| BooleanPolicy | missing (Agent F) |
| ResolutionProfile | C-11 |
| CompositeReference | C-14 |
| PersistenceCommit | P-05 StorageTransaction |
