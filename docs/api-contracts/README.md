# Cube — API Contracts Index (Geometry)

**Status:** DRAFT 0.1. **No contract in this directory is approved.** Approval is recorded by the Integration Owner in `docs/decisions/`.
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

**Counting note (official wording):** the set is **45 contracts + 3 interface boundaries** (PR-09, E-13, P-09; boundary-only, no version yet), plus 2 unnumbered L2 storage ports (`IDeltaStore`, `ISnapshotStore`). The phrase "48 contracts" is not used: it counted the boundaries as contracts. Section ids E-01 (pointer to C-17) and P-08 (composite persistence note) are not contracts and carry no row. Plus two storage ports (`IDeltaStore`, `ISnapshotStore`) listed unnumbered. **Owner column = the single contract owner**; the agent that implements an operation or algorithm over the data is named in the consumers column.

Version scheme: `major.minor.patch[-draft]`. All contracts are `0.1.0-draft`; `1.0.0` is assigned on approval. Per-contract ownership, lifetime, access, thread-safety, revision semantics, errors and contract tests are in the file named in the *Spec file* column.

Legend: **Draft** = specified in this pass, awaiting approval. **Draft+OD** = specified, with named Open Decisions that block freezing. **Boundary** = only the interface boundary is specified.

| ID | Contract | Provider (owner) | Primary consumers | Spec file | State | Version | Blocking open decisions |
|---|---|---|---|---|---|---|---|
| C-01 | GeometryAsset | Core | all | core | Draft | 0.1.0-draft | metadata schema |
| C-02 | GeometryLayer | Core | all | core | Draft+OD | 0.1.0-draft | collision profile schema |
| C-03 | GeometryLayerType | Core | API, backends, tools | core | Draft+OD | 0.1.0-draft | extension registration |
| C-04 | GeometryTransform | Core | all | core | Draft+OD | 0.1.0-draft | f32/f64, epsilon |
| C-05 | GeometryBounds | Core | Processing (derived bounds, PR-08), backends | core | Draft+OD | 0.1.0-draft | bounds exactness |
| C-06 | GeometryRegion | Core | all | core | Draft+OD | 0.1.0-draft | non-AABB shapes |
| C-07 | BrickKey | Core | Voxel, Persistence, all | core | Draft+OD | 0.1.0-draft | int width, refinement addressing |
| C-08 | BrickStorage | Core | Processing, backends, Persistence | core | Draft+OD | 0.1.0-draft | layout, index order, compression |
| C-09 | GeometryChannel | Core | backends, tools | core | Draft+OD | 0.1.0-draft | attribute registry, widths |
| C-10 | MaterialMapping | Core | backends, Boolean, Sculpt | core | Draft+OD | 0.1.0-draft | global library interface |
| C-11 | ResolutionProfile | Core | backends, Resolution | core | Draft | 0.1.0-draft | value sets |
| C-12 | RefinementRegion | Core | backends, Resolution Agent (operations) | core | Draft+OD | 0.1.0-draft | transition algorithm |
| C-13 | Junction | Core | Boolean, Processing | core | Draft | 0.1.0-draft | blend params |
| C-14 | CompositePart | Core | Composite Agent (operations), Persistence | core | Draft+OD | 0.1.0-draft | ACR-003 |
| C-15 | AssemblyGraph | Core | Composite Agent (operations), Persistence | core | Draft+OD | 0.1.0-draft | ACR-003, constraint scope |
| C-16 | RevisionToken | Core | all | core + semantics | Draft+OD | 0.1.0-draft | OD-R1…R8 |
| C-17 | OperationId | Core | all | core | Draft+OD | 0.1.0-draft | OD-R8 |
| C-18 | DirtyRegion | Core | Processing, Orchestration, backends | core | Draft+OD | 0.1.0-draft | sub-brick granularity |
| C-19 | Result/Error envelope | Core | all | core | Draft | 0.1.0-draft | code scheme |
| PR-01 | Processing Snapshot | Processing | Orchestration, backends, Boolean, Persistence | processing | Draft+OD | 0.1.0-draft | pin vs copy, caps |
| PR-02 | Dependency Revision | Processing | Processing, Orchestration | processing | Draft+OD | 0.1.0-draft | granularity |
| PR-03 | Processing Task / AsyncJobHandle | Processing | Orchestration, Tools, Persistence | processing | Draft+OD | 0.1.0-draft | task kind extension |
| PR-04 | Processing Result | Processing | Orchestration | processing | Draft+OD | 0.1.0-draft | payload typing |
| PR-05 | CancellationToken | Processing | all async | processing | Draft | 0.1.0-draft | deadlines |
| PR-06 | Job Priority | Processing | submitters | processing | Draft+OD | 0.1.0-draft | weights |
| PR-07 | Dependency Graph registration | Processing | derived-data owners | processing | Draft+OD | 0.1.0-draft | provider ownership (O-1) |
| PR-08 | Spatial Hierarchy / Bounds / BVH update | Processing | queries, collision | processing | Draft+OD | 0.1.0-draft | query API home |
| PR-09 | Engine ports | Processing | Engine adapters | processing | Boundary | — | exact set |
| E-02 | AuthorizationContext | Editing API (input from session layer) | Orchestration, Networking | editing | Draft+OD | 0.1.0-draft | permission model |
| E-03 | EditingOperation | Editing API | Orchestration, Tools, History | editing | Draft+OD | 0.1.0-draft | Kind registry |
| E-04 | EditingTransaction | Editing API | Orchestration (implements), Tools, backends, History | editing | Draft+OD | 0.1.0-draft | nested/group, async |
| E-05 | EditingResult | Editing API | all | editing | Draft | 0.1.0-draft | — |
| E-06 | ConflictResult | Editing API | Orchestration, Networking | editing | Draft+OD | 0.1.0-draft | merge policy |
| E-07 | IGeometryBackend / Capabilities | Editing API | Orchestration, backends, Tools | editing | Draft+OD | 0.1.0-draft | capability vocabulary |
| E-08 | ModifierStack | Editing API | Tools, evaluation | editing | Draft+OD | 0.1.0-draft | evaluation owner (O-2) |
| E-09 | GeometryDelta | Editing API | History, Persistence | editing | Draft+OD | 0.1.0-draft | delta encoding |
| E-10 | Checkpoint | Editing API | History, Persistence | editing | Draft+OD | 0.1.0-draft | frequency policy |
| E-11 | IEditingService | Editing API | Tools, Composite, History, Networking | editing | Draft+OD | 0.1.0-draft | signatures |
| E-12 | ITransactionObserver | Editing API | Orchestration (publish), History (sink) | editing | Draft+OD | 0.1.0-draft | durability ordering |
| E-13 | Networking boundary | Editing API | Networking Adapter | editing | Boundary | — | owner, protocol (O-3) |
| — | IDeltaStore / ISnapshotStore | Editing API | History, Persistence | editing §0 | Draft | 0.1.0-draft | — |
| P-01 | AssetManifest | Persistence | Persistence, tools | persistence | Draft+OD | 0.1.0-draft | container, format |
| P-02 | BrickSerialization | Persistence | Persistence | persistence | Draft+OD | 0.1.0-draft | compression |
| P-03 | DeltaSerialization | Persistence | History (via port) | persistence | Draft+OD | 0.1.0-draft | flush policy |
| P-04 | SnapshotSerialization | Persistence | History (via port) | persistence | Draft+OD | 0.1.0-draft | incremental |
| P-05 | StorageTransaction | Persistence | Persistence | persistence | Draft+OD | 0.1.0-draft | platform atomic publish |
| P-06 | IntegrityReport | Persistence | Core validation, QA | persistence | Draft | 0.1.0-draft | repair automation |
| P-07 | MigrationDescriptor | Persistence | Persistence | persistence | Draft | 0.1.0-draft | execution mode |
| P-09 | Import/Export adapters | unassigned | — | persistence | Boundary | — | owner (O-4) |
| — | BooleanPolicy | Agent F | Orchestration, backends | *not yet written* | **Missing** | — | next pass |
| — | BrushParameters / Sculpt tool interface | Agent E | Orchestration, backends | *not yet written* | **Missing** | — | next pass |
| — | Resampling/Refinement operation contract | Resolution Agent | backends | *not yet written* | **Missing** | — | next pass |
| — | Composite operation contract | Composite Agent | Orchestration | *not yet written* | **Missing** | — | next pass |
| — | History interface (undo/redo/limits) | Agent G | UI, Networking | *not yet written* | **Missing** | — | next pass |

Tool-level contracts (last five rows) were outside the minimum list requested for this pass and belong to their module owners after the P0–P2 contracts above are approved.

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
