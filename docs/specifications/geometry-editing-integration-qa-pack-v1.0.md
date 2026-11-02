# Cube — Geometry Editing Integration & QA Pack v1.0

**Document type:** Integration, Dependency and Quality Assurance Specification\
**Scope:** Geometry Editing v1.0\
**Owner:** Integration Owner\
**Revision:** 1.1 — aligned with Architecture Update v1.0 (AU-1.0). Supersedes the v1.0 dependency matrix, contract matrix and build pipeline.\
**Related:** `docs/api-contracts/README.md`, `docs/api-contracts/revision-and-snapshot-semantics.md`

---

# 1. Module Dependency Matrix

Rules (AU-1.0 §3): dependencies are strictly one-directional by layer; lower layers never depend on upper layers; no cycles; modules interact only through approved API Contracts; QA is a test-time dependency only. The matrix describes **compile-time** dependencies. It does not authorize access to internals.

**Superseded matrix:** the v1.0 dependency matrix (which contained a History ↔ Persistence cycle and an Orchestration ↔ History cycle, and gave QA a Required dependency on every module) is **Deprecated/historical**. It is no longer part of this document and survives only in git history (commit `dfba948`). Sections 1.1–1.5 are the only valid dependency definition.

## 1.1 Layers

| Layer | Module | Owner |
|---|---|---|
| L0 | Geometry Core | Core Agent |
| L1 | Processing | Processing Agent |
| L2 | Editing API & Contracts (contracts, facade interface, ports) | Main Agent / Integration Owner |
| L3 | Editing Orchestration | Agent A |
| L4 | Geometry Backends: Voxel / SDF / Mesh | Agents B / C / D |
| L5 | Editing Tools: Sculpting / Boolean / Resampling & Refinement | Agents E / F / Resolution Agent |
| L6 | History | Agent G |
| L7 | Composite Geometry | Composite Agent |
| L8 | Persistence Integration | Agent H |
| L9 | Networking Adapter | **unassigned (Open O-3)** |
| L10 | Engine Integration Adapters | **unassigned (Open O-5)** |
| — | QA & Benchmarks (test-time only) | Agent I |

L3–L8 are **peers above L2** with respect to each other as shown in the grid; the layer numbers fix reading order, not permission to depend on every lower number. Where the grid shows no entry, no dependency is allowed.

## 1.2 Compile-time dependency grid

Row = consumer. `D` = direct dependency on the provider's public API. `A` = abstraction only (consumes a port/contract defined in L2, never the implementation). Blank = forbidden.

| Consumer \ Provider | Core | Proc | API | Orch | Backends | Sculpt | Boolean | Resample | History | Composite | Persist | Net | Engine |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Core | — | | | | | | | | | | | | |
| Processing | D | — | | | | | | | | | | | |
| Editing API | D | D | — | | | | | | | | | | |
| Orchestration | D | D | D | — | A (via `IGeometryBackend`) | | | | A (`ITransactionObserver`) | | | | |
| Voxel / SDF / Mesh | D | D | D | | — | | | | | | | | |
| Sculpting | D | D | D | A (`IEditingService`) | A (capabilities) | — | | | | | | | |
| Boolean | D | D | D | A (`IEditingService`) | A (capabilities) | | — | | | | | | |
| Resampling & Refinement | D | D | D | A (`IEditingService`) | A (capabilities) | | | — | | | | | |
| History | D | D | D | A (`IEditingService`) | | | | | — | | A (`IDeltaStore`, `ISnapshotStore`) | | |
| Composite | D | D | D | A (`IEditingService`) | | | | | | — | | | |
| Persistence | D | D | D | | | | | | | | — | | |
| Networking Adapter | D (ids/tokens only) | | D | A (`IEditingService`) | | | | | | | | — | |
| Engine Adapters | D (ports) | D (ports) | | | | | | | | | | | — |
| QA (test-time) | D | D | D | D | D | D | D | D | D | D | D | D | D |

Notes on the grid:

1. **Orchestration → Backends** is `A`: Orchestration holds a registry of `IGeometryBackend` (defined in L2). Backends depend on L2 only. Orchestration never imports backend implementation.
2. **History ↔ Persistence:** History uses `IDeltaStore`/`ISnapshotStore` (defined in L2); Persistence implements them using `GeometryDelta`/`Checkpoint` (L2 data contracts). Neither imports the other. A runtime path History → port → Persistence exists; the compile-time graph is acyclic.
3. **Orchestration → History** is `A`: Orchestration publishes to `ITransactionObserver`; History implements it and applies undo through `IEditingService`. Both point at L2, so no cycle.
4. **Backends do not call Tools** (grid blank). Tools reach backends only through capabilities.
5. **Persistence does not depend on History, Orchestration or Backends.** It can be developed in parallel with them once Core, Processing and L2 contracts are frozen.
6. **Engine Adapters implement ports** defined in Core/Processing (inversion); Core and Processing do not depend on any engine.
7. **Composite does not merge Parts**; Boolean/Resample operations on composite parts are separate explicit operations.
8. The **composition root** (wiring of Orchestration + backends + History + Persistence + Networking) is owned by the Integration Owner and is not part of any module.

## 1.3 Provider / Consumer / Interface table

| # | Provider | Consumer | Interface | Direction | Ownership of data | Data lifetime | Revision requirement | Threading requirement |
|---|---|---|---|---|---|---|---|---|
| 1 | Core | Processing | read views, `BrickStorage` pins, Bounds, RevisionToken, DirtyRegion, Result/Error | Proc→Core | Core owns payload | pin ref-counted | tokens compared, never produced | concurrent readers |
| 2 | Core | Editing API | all Core contracts (C-01…C-19) | API→Core | Core | by handle | tokens in results | value-safe |
| 3 | Processing | Editing API | Snapshot, Task/AsyncJobHandle, CancellationToken | API→Proc | Processing | task lifetime | DependencyRevision | thread-safe handles |
| 4 | Editing API | Orchestration | `IEditingService` (implements), operation/transaction/result contracts | Orch→API | Orch owns live transactions | transaction scope | commit protocol (semantics §4.1) | single writer per transaction; disjoint transactions concurrent |
| 5 | Core | Orchestration | commit primitive on `BrickStorage` | Orch→Core | Core | atomic | sole revision writer is Core commit | per-Asset commit gate |
| 6 | Editing API | Backends | `IGeometryBackend`, `BackendCapabilities`, overlay/WriteSet | Backend→API | overlay private to transaction | transaction | backend reports actual DirtyRegion | workers on snapshots |
| 7 | Orchestration | Backends | (none directly) registry through L2 | — | — | — | — | — |
| 8 | Editing API | Tools | `IEditingService`, operation Kind schemas | Tool→API | tools own their Kind schemas | operation | tokens via transaction | stroke = one transaction (Open) |
| 9 | Processing | Tools (Boolean, Resample) | Snapshot + Task handle | Tool→Proc | Processing | task | revalidate at commit | async + cancel |
| 10 | Editing API | History | `ITransactionObserver`, `GeometryDelta`, `Checkpoint`, `IEditingService` | Hist→API | History owns entries; payload in COW/store | per retention | Pre/PostTokens | ordered per Asset |
| 11 | Editing API | Persistence | `IDeltaStore`, `ISnapshotStore`, `GeometryDelta`, `Checkpoint` | Pers→API | Persistence owns bytes | durable | tokens/Epoch persisted | async writers |
| 12 | Core | Persistence | export/import views of Bricks and manifest data | Pers→Core | Core owns runtime, Persistence owns format | serialize/deserialize | brick revision + hash | background |
| 13 | Processing | Persistence | jobs, snapshots, cancellation | Pers→Proc | Processing | save duration | consistent cut | async |
| 14 | Core | Composite | `CompositePart`, `AssemblyGraph` data (ACR-003) | Comp→Core | Core data / Composite ops | with asset | Asset revision | via transactions |
| 15 | Editing API | Composite | `IEditingService` | Comp→API | Composite owns constraint logic | operation | tokens | via transactions |
| 16 | Editing API | Networking | `IEditingService`, `AuthorizationContext`, `EditingResult`, `ConflictResult` | Net→API | server authoritative | message | ExpectedTokens, OperationId | I/O threads separate from game thread |
| 17 | Core/Processing | Engine Adapters | ports (job executor, allocator, clock) | Adapter implements | engine side owns resources | process | n/a | engine-specific |
| 18 | all modules | QA | public contracts only | QA→all | QA owns tests | test run | n/a | n/a |

## 1.5 Dependency-direction rule DEP-01 and its test (specification only)

**Rule DEP-01.** Source-level imports between modules are allowed only where the grid in §1.2 shows `D`; cells marked `A` permit an import of the L2 port contract only, never of the provider module. The compile-time graph must be acyclic.

**Required automated test (not yet implemented; blocks closing B-9):**
1. Parse the machine-readable form of §1.2 (to be produced with the repository tooling once the language is chosen).
2. Fail on any import not allowed by the grid.
3. Fail on any cycle (self-test with an injected History→Persistence import must fail).
4. Fail if a module marked `A` imports a provider implementation.
5. Runtime loops through ports (Orchestration ⇄ History) are not detected by this test; they are covered by integration tests for ordered, non-reentrant observer delivery.

Status: rule recorded; test **absent**.

## 1.4 Acyclicity argument

Edges only go to lower layers or to L2 abstractions: L0←L1←L2←{L3, L4, L5, L6, L7, L8, L9}; L5/L6/L7/L9 consume L3 only as `IEditingService` (defined in L2). The compile-time graph is acyclic (checked once by script; DEP-01 will enforce it continuously). Runtime back-paths through ports exist and are intentional (see audit report §A.1). A topological order exists: L0, L1, L2, then {L3, L4, L8} in parallel, then {L5, L6, L7} in parallel, then L9, L10. CI shall enforce this with an automated dependency-direction check (Section 6).

# 2. API Contract Matrix

The authoritative matrix lives in `docs/api-contracts/README.md` §3 (contracts C-01…C-19, PR-01…PR-09, E-02…E-13, P-01…P-09, each with provider, consumers, state and blocking open decisions). All contracts are currently **Draft**; none is approved. Name mapping from the former v1.0 matrix is in the same file (§4).

Ownership corrections made by AU-1.0 alignment (single owner each):

- `RevisionToken`, `OperationId`, `DirtyRegion`, Result/Error envelope: **Core** (foundation types; ACR-001). Previously "Core / Processing" or Editing.
- `GeometrySnapshot`: **Processing** (`ProcessingSnapshot`), over Core COW pins.
- `CompositeReference` → `CompositePart` / `AssemblyGraph`: data in Core, operations in Composite (ACR-003).
- `PersistenceCommit` → `StorageTransaction`: **Persistence**; History uses `IDeltaStore`/`ISnapshotStore` instead.

# 3. Required API Contract Details

Detailed contract specifications moved to `docs/api-contracts/`. This section keeps only what QA must verify for the cross-cutting contracts:

- **RevisionToken / Snapshot / Dirty:** `docs/api-contracts/revision-and-snapshot-semantics.md` (tests §8 there are mandatory QA suites).
- **AsyncJob states** (Processing Task): `Created, Queued, Running, Completed, Failed, Cancelled, Stale`; terminal states are final; `Stale` = finished but rejected at publish.
- **GeometryOperationResult:** variants Success, ValidationFailure, Conflict, Unsupported, ConversionRequired, ResourceFailure, Cancelled; partial results only with an explicit per-operation contract.
- **EditingTransaction:** identity, targets, read/write regions, expected tokens, operation sequence, commit/rollback policy, authorization context.

# 4. Module Ownership Matrix

One owner per module (AU-1.0 §2).

| Module | Primary Owner | Forbidden Responsibilities |
|---|---|---|
| Geometry Core | Core Agent | Engine/UI/network/persistence dependencies; editing logic; Processing dependency |
| Processing | Processing Agent | Editing or gameplay business logic; mutating authoritative geometry |
| Editing API & Contracts | Main Agent / Integration Owner | Unreviewed contract changes; backend algorithms |
| Orchestration | Agent A | Backend algorithms; history/persistence internals |
| Voxel Backend | Agent B | SDF/Mesh implementation; calling Tools |
| SDF Backend | Agent C | Voxel/Mesh implementation; calling Tools |
| Mesh Backend | Agent D | Voxel/SDF implementation; calling Tools |
| Sculpting | Agent E | Owning backend storage |
| Boolean | Agent F | Owning global geometry storage |
| Resampling & Resolution Refinement | Resolution Agent | Owning brick storage; implicit conversion |
| History | Agent G | Own Brick Storage; bypassing transactions; importing Persistence internals |
| Composite Geometry | Composite Agent | Automatic merging of Parts |
| Persistence Integration | Agent H | Replacing Core storage; controlling editing; importing History internals |
| Networking Adapter | **Unassigned (Open O-3)** | Transporting render meshes as truth |
| Engine Integration Adapters | **Unassigned (Open O-5)** | Leaking engine types into Core/Processing |
| QA & Benchmarks | Agent I | Changing production behavior to hide test failures; runtime dependency |
| Shared Contracts | Integration Owner | Unreviewed changes |

# 5. Layered Build Pipeline

Order follows AU-1.0 §26. The dependency grid (Section 1.2) allows overlap: the parallelism column states what may start early. A stage's **exit condition** is mandatory before dependent stages are accepted.

| Stage | Module(s) | Validation | Exit condition | May start in parallel once |
|---|---|---|---|---|
| 0 | Decisions & tier approval | language decision recorded (AU-1.0 §24 #1; the engine decision is needed only for stage 12); **P0 approved** (C-01…C-19, gates stage 1); P1 under review; rule DEP-01 recorded (its test is required before stage 1 acceptance); tiers defined in `docs/api-contracts/README.md` §3 | Integration Owner approval of P0 | — |
| 1 | Geometry Core | asset/layer model, Bounds, Region, Transform, Brick addressing (negative coords), BrickStorage with COW and missing-state distinction, channels, material mapping, revision primitives and commit primitive, brick export/import views (**not** a file format) | Core tests pass | stage 0 done (P0 approved) |
| 2 | Processing foundations | snapshot capture, task state machine, scheduler, cancellation, dependency graph, revision validation, stale rejection, bounds/BVH slots | Concurrency and async tests pass | **P1 approved** and stage 1 contracts frozen |
| 3 | Editing API & Contracts | public types, error model, ports (`IEditingService`, `IGeometryBackend`, `ITransactionObserver`; the storage ports `IDeltaStore`/`ISnapshotStore` are P2), threading/revision/async docs; finalises P1 | Contract tests pass | stages 1–2 contract drafts; P1 approved |
| 4 | Editing Orchestration | transaction lifecycle, backend registry/dispatch, rollback, dirty propagation, revision validation, conflicts | Orchestration integration tests pass | stage 3 frozen |
| 5 | Backends (Voxel, SDF, Mesh) | conformance suite for `IGeometryBackend`, backend-specific correctness | all required backend tests pass | stage 3 frozen (use orchestration test harness before stage 4 completes) |
| 6 | Tools: Sculpting, Boolean | transaction integration, explicit conversion, async revision safety | Tool integration tests pass | stage 4 + 5 usable |
| 7 | History | commands/deltas/snapshots/undo/redo, memory limits, local vs server history | History restoration tests pass | stage 4 and the **P2 subset** (E-10, `IDeltaStore`, `ISnapshotStore`) approved |
| 8 | Composite Geometry | hierarchy, assembly graph, attach/detach, constraints, transform propagation | Composite integration tests pass | stage 4 |
| 9 | Resampling & Refinement | resolution change, refinement regions, transitions, undo | Mixed-resolution tests pass | stage 5 |
| 10 | Persistence Integration | manifest, bricks, deltas, snapshots, atomic save, recovery, migration | Persistence and recovery tests pass | stages 1–3 and the **P2 subset** (P-01…P-07, storage ports) approved (does not need 4–9; full gate needs 7–8) |
| 11 | Networking Adapter | server authority, reconciliation, reconnect, journal | Multiplayer tests pass | stages 4, 7, 10; **owner unassigned** |
| 12 | Engine Integration Adapters | port implementations only | Adapter conformance tests pass | any time after stage 2; **owner unassigned**, engine not chosen |
| 13 | Full Integration & QA | complete dependency graph, all contracts, full suite, stress, baselines, regression | Integration Owner approval | all |

Tier gates: P0 → stage 1; P1 → stages 2–5; P2 → the module it affects (stages 6–10); P3 → stages 11–12 and tool modules. QA infrastructure (Agent I) begins at stage 0 and must exist before stage 1 acceptance.

# 6. CI Pipeline

Recommended stages:

```
Pull Request
    |
    v
Formatting / Static Analysis
    |
    v
Compile
    |
    v
API Contract Validation
    |
    v
Unit Tests
    |
    v
Module Tests
    |
    v
Integration Tests
    |
    v
Regression Tests
    |
    v
Performance Smoke Tests
    |
    v
Review
    |
    v
Merge Approval
```

Full stress and extended benchmarks may run on scheduled CI jobs or dedicated benchmark infrastructure.

Required CI checks must not be silently skipped.

Additional required CI checks (AU-1.0 alignment):

- **Dependency-direction check:** fails on any import not allowed by Section 1.2 and on any cycle.
- **Engine-independence check:** Core and Processing contain no engine or network dependencies.
- **Contract-version check:** a changed contract file without an approval record in `docs/decisions/` fails the build.

# 7. QA Test Matrix

| Test Category  | Voxel            | SDF              | Mesh             | Tools    | History  | Persistence |
| -------------- | ---------------- | ---------------- | ---------------- | -------- | -------- | ----------- |
| Unit           | Required         | Required         | Required         | Required | Required | Required    |
| Module         | Required         | Required         | Required         | Required | Required | Required    |
| Integration    | Required         | Required         | Required         | Required | Required | Required    |
| Property-Based | Required         | Required         | Required         | Required | Required | Required    |
| Regression     | Required         | Required         | Required         | Required | Required | Required    |
| Performance    | Required         | Required         | Required         | Required | Required | Required    |
| Stress         | Required         | Required         | Required         | Required | Required | Required    |
| Determinism    | Where applicable | Where applicable | Where applicable | Required | Required | Required    |

Modules added by AU-1.0 follow the same rule: **Geometry Core, Processing, Resampling & Refinement and Composite** require every category above except where marked "Where applicable"; Core and Processing additionally require Concurrency and Revision-semantics suites (`docs/api-contracts/revision-and-snapshot-semantics.md` §8). The **Networking Adapter** requires the multiplayer suite. Engine Adapters require port-conformance tests only.

Multiplayer tests must additionally cover transaction conflicts, concurrent region edits, server rejection and reconnect.

# 8. Geometry Correctness Test Scenarios

## 8.1 Brick Boundaries

Test:

- Positive coordinates.
- Negative coordinates.
- Brick transitions.
- Corner and edge cases.
- Missing neighbors.
- Empty neighbors.
- Unloaded neighbors.

Expected:

- No address corruption.
- No incorrect boundary modifications.
- Correct missing-data behavior.

## 8.2 Mixed Resolution

Test:

- Equal-resolution boundaries.
- Different refinement levels.
- Refinement creation.
- Refinement removal.
- Resampling.
- Repeated refinement changes.

Expected:

- No visible cracks.
- Correct transition geometry.
- Stable material mapping.
- Correct invalidation.

## 8.3 Transactions

Test:

- Commit.
- Rollback.
- Validation failure.
- Revision conflict.
- Cancellation.
- Partial failure.

Expected:

- No unauthorized partial authoritative state.
- Correct history behavior.
- Correct revision handling.

## 8.4 Async Processing

Test:

- Valid snapshot.
- Stale snapshot.
- Dependency changed during execution.
- Cancellation.
- Worker failure.
- Priority starvation.

Expected:

- Stale results are never published.
- Resources are released.
- Valid jobs eventually complete.

## 8.5 Boolean

Test:

- Empty operands.
- Identical operands.
- Disjoint operands.
- Contained operands.
- Intersecting operands.
- Degenerate operands.
- Mixed representations.
- Conversion failures.

Expected:

- Correct operation semantics.
- Explicit conversion.
- Defined material behavior.
- Transactional result.

## 8.6 Undo / Redo

Test:

- Single edit.
- Multiple edits.
- Large edit.
- Boolean result.
- Sculpting stroke.
- Resolution change.
- Save and reload.
- History limit reached.

Expected:

- Correct restoration.
- Correct dependency invalidation.
- No memory growth beyond configured limits.

# 9. Performance Benchmark Specification

Numeric budgets must be established from measured baselines.

## 9.1 Benchmark Dimensions

| Dimension   | Test Profiles                  |
| ----------- | ------------------------------ |
| Asset size  | Small / Medium / Large         |
| Brick count | Low / Medium / High            |
| Occupancy   | Sparse / Mixed / Dense         |
| Geometry    | Voxel / SDF / Mesh / Composite |
| Resolution  | Base / Refined / Mixed         |
| Editing     | Local / Regional / Large       |
| Concurrency | Single / Multiple workers      |
| History     | No history / Delta / Snapshot  |
| Persistence | Save / Load / Recovery         |

## 9.2 Metrics

- Median CPU time.
- High-percentile CPU time.
- Peak memory.
- Allocation volume.
- Main-thread blocking.
- Async queue latency.
- Processing throughput.
- Cache hit rate where applicable.
- Scaling behavior.

## 9.3 Performance Report

Each report must include:

- Dataset ID.
- Dataset description.
- Build identifier.
- Hardware.
- Software configuration.
- Repetition count.
- Warm-up method.
- Measurement method.
- Results.
- Comparison to baseline.
- Known limitations.

## 9.4 Platform Coverage

Eventually benchmark:

- Desktop High-End.
- Desktop Mid-Range.
- Mobile High-End.
- Mobile Low-End.

The initial prototype may use a smaller hardware matrix, but the final target matrix must include representative mobile devices.

Long-duration tests must include thermal throttling and memory pressure.

# 10. Integration Checklist

## Before Integration

- Module owner assigned.
- API Contract approved.
- Dependencies declared.
- Module compiles independently.
- Unit tests pass.
- Module tests pass.
- Documentation updated.
- No unauthorized API changes.
- No unrelated modifications.

## During Integration

- Public API compatibility verified.
- Dependency graph validated.
- No cyclic dependencies.
- Revision behavior verified.
- Dirty Tracking verified.
- Async ownership verified.
- Error propagation verified.
- Memory lifetime verified.
- Integration tests pass.

## Before Merge

- Regression tests pass.
- Required benchmarks completed.
- No critical defects.
- Known limitations documented.
- Integration Owner approves.

# 11. Strict Acceptance Checklist

A module is rejected if any mandatory criterion fails.

## Functional

- Approved scope implemented.
- Required operations implemented.
- Edge cases covered.
- Error behavior correct.

## Architecture

- Strict API Contracts respected.
- Module ownership respected.
- No hidden dependencies.
- No unauthorized architecture changes.

## Data Integrity

- Revision consistency.
- Dirty Tracking.
- Transactional behavior.
- Correct resource ownership.
- No data corruption.

## Async and Concurrency

- Immutable snapshots.
- Revision validation.
- Cancellation.
- Thread safety.
- Stale result rejection.

## Quality

- Tests pass.
- Regression tests pass.
- Performance report delivered.
- Documentation complete.

## Integration

- Dependencies verified.
- Compatibility verified.
- CI passes.
- Integration Owner approves.

# 12. Release Readiness

Geometry Editing v1.0 is ready for release only when:

1. All mandatory modules are accepted.
2. All contracts are approved and implemented.
3. Full integration pipeline passes.
4. No critical data corruption issues remain.
5. Persistence recovery is verified.
6. Async revision safety is verified.
7. Undo / Redo is verified.
8. Multiplayer concurrency is verified.
9. Performance baselines are documented.
10. Required target platforms are tested.
11. Documentation is complete.
12. Integration Owner approves the release.

# 13. Known Limitations and Deferred Decisions

Authoritative list of open decisions: `docs/decisions/audit-resolution-report.md` §H (consolidating AU-1.0 §24, the Open items of every contract file and the new items O-1…O-6). Still implementation-dependent:

- Programming language and engine.
- Integer and floating-point types.
- Formal RevisionToken details and atomic commit algorithm (proposal in `revision-and-snapshot-semantics.md`).
- Exact API signatures.
- Snapshot/COW implementation, conflict-resolution algorithm.
- Boolean and refinement-transition algorithms.
- SDF storage precision, compression, serialization format.
- Network protocol and transport.
- Numeric performance budgets, benchmark infrastructure, CI provider.

These must be resolved through implementation evidence and approved technical decisions.

**End of Integration & QA Pack.**
