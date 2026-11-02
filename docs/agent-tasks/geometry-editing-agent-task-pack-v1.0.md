# Cube — Geometry Editing Agent Task Pack v1.0

**Document type:** Agent Work Assignment Package\
**Scope:** Geometry Editing v1.0\
**Coordination:** Central Integration\
**Contract authority:** Integration Owner\
**Revision:** 1.1 — aligned with Architecture Update v1.0 (AU-1.0). Adds Core, Processing, Composite and Resolution agents; replaces the dependency diagram and execution order.

---

# 1. General Instructions

All agents must read:

- GDD/TDD v0.3.
- Voxel Architecture v1.0.
- Geometry Processing decisions.
- Architecture Update v1.0 (AU-1.0).
- Geometry Editing Specification v1.0 (rev 1.1).
- API Contracts index: `docs/api-contracts/README.md` and the file for each contract you consume.
- Cube Agent Development Specification v1.0.
- Their individual task below.

Do not independently redefine:

- Geometry Asset.
- Geometry Layer.
- Brick addressing.
- Attribute Channels.
- Revision semantics.
- Dirty Tracking.
- Transaction semantics.
- Public error types.
- Async job lifecycle.

These belong to shared contracts.

If a required interface is missing, submit an API Contract Change Request.
Each agent works only against contracts in state **Approved**. Draft contracts may be used for planning and test design, not for shared implementation.
No agent implements a hidden replacement for a module that has no implementation yet (AU-1.0 §2.1 rule 8).

# 2. Task Dependency Overview

Dependencies follow the Integration & QA Pack dependency grid (Section 1.2). `A` means abstraction only: consume the port from Editing API & Contracts, never the implementation.

```
Core Agent (L0)
   |
Processing Agent (L1)
   |
Editing API & Contracts (L2, Integration Owner)
   |
   +-- Agent A: Orchestration (L3) ----+
   +-- Agents B, C, D: Backends (L4)   |  consume L2 ports only
   +-- Agent H: Persistence (L8) ------+  (needs Core, Processing, L2; not History/Orchestration)
   |
   +-- after Orchestration + >=1 backend usable:
   |     Agent E Sculpting, Agent F Boolean, Resolution Agent (L5)
   |     Agent G History (L6)   Composite Agent (L7)
   |
   +-- Networking Adapter (L9, owner unassigned)
   +-- Engine Adapters (L10, owner unassigned)

Agent I (QA): test-time dependency on everything; starts at stage 0
```

| Agent | Provides | Depends on (D = direct, A = abstraction) |
|---|---|---|
| Core Agent | Core contracts C-01…C-19 | none |
| Processing Agent | PR-01…PR-09 | Core (D) |
| Integration Owner | Editing contracts E-xx, ports | Core (D), Processing (D) |
| A Orchestration | `IEditingService` implementation | Core, Processing, Editing API (D); Backends (A); History (A) |
| B, C, D Backends | `IGeometryBackend` implementations | Core, Processing, Editing API (D) |
| E Sculpting | sculpt operation Kinds | Core, Processing, Editing API (D); Orchestration (A); Backends (A, capabilities) |
| F Boolean | boolean operation Kinds, BooleanPolicy | same as E |
| Resolution Agent | resample/refinement Kinds | same as E |
| G History | `ITransactionObserver` implementation | Core, Processing, Editing API (D); Orchestration (A, undo via `IEditingService`); Persistence (A, `IDeltaStore`/`ISnapshotStore`) |
| Composite Agent | composite operation Kinds | Core, Processing, Editing API (D); Orchestration (A) |
| H Persistence | implementations of `IDeltaStore`, `ISnapshotStore`, manifest/brick format | Core, Processing, Editing API (D) |
| I QA | test infrastructure, benchmarks | all (test-time only) |

This is the logical dependency picture. Actual execution may overlap once the contracts each agent needs are Approved.

# 2.5 Foundation Agents (new in rev 1.1)

## Core Agent — Geometry Core

**Primary responsibility:** Geometry Core (L0). **Contracts owned:** C-01…C-19.

**Objective:** implement the engine-independent data foundation that every other module builds on. This module did not exist in the repository when Task Pack v1.0 was written.

**Scope:** Asset/Layer model; layer types and registry; Transform; Bounds (Local AABB, derived types, Empty); Region; BrickKey and addressing (signed coordinates); BrickStorage (sparse, hash-map access, octree organisation, copy-on-write, pins, missing-data states); channels (adaptive storage, categorical vs continuous); material mapping; ResolutionProfile; Refinement/Junction/Composite **data models**; RevisionToken, OperationId, DirtyRegion, Result/Error envelope; the commit primitive; brick export/import views.

**Must not:** depend on engine, UI, network, Processing or Persistence; define a file format; implement editing algorithms.

**Tests:** addressing at boundaries and negative coordinates; COW isolation; missing-state distinction; revision hierarchy invariant; transform properties.

**Acceptance:** Core contracts implemented as approved; zero outbound dependencies except the language runtime.

## Processing Agent — Processing

**Primary responsibility:** Processing foundations (L1). **Contracts owned:** PR-01…PR-09.

**Scope:** snapshot capture and lifetime; task state machine; priority scheduler with starvation prevention; cancellation; dependency graph and invalidation by ChangeType; revision validation and stale rejection; Dirty Region expansion; bounds/BVH/LOD slots; engine ports definition.

**Must not:** contain editing or gameplay logic; mutate authoritative geometry; implement mesh/collision algorithms unless an owner is assigned (Open O-1).

**Tests:** stale results never published; cancellation releases pins; no starvation; invalidation matrix.

**Acceptance:** concurrency and async suites pass; revision semantics tests pass.

## Composite Agent — Composite Geometry

**Primary responsibility:** hierarchy, assembly graph and part operations (L7).

**Scope:** Attach, Detach, Reparent, Transform, Apply Constraints, Destroy Part — all as operations through `IEditingService`; transform propagation; attachment points; constraint evaluation.

**Must not:** merge part geometry automatically; own the part/graph data model if ACR-003 places it in Core.

**Tests:** hierarchy and graph integrity, cycle rejection, propagation, destroy cascade, reference restoration (with Persistence).

**Acceptance:** composite integration tests pass; references survive reload.

## Resolution Agent — Resampling & Resolution Refinement

**Primary responsibility:** resolution changes (L5, Editing Tools).

**Scope:** Resample Layer under explicit policy (shape, materials, attributes); create/remove Refinement Regions; transition handling without cracks; invalidation; undo-compatible results; heavy-operation locks (with Orchestration).

**Must not:** own brick storage; convert geometry types implicitly; choose a transition algorithm before it is approved (Open).

**Tests:** equal-resolution and mixed-level boundaries, repeated refine/unrefine, resampling fidelity, material mapping stability.

**Acceptance:** mixed-resolution scenarios of QA Pack §8.2 pass.

# 3. Agent A — Editing Orchestration

**Depends on:** Core, Processing, Editing API (D); Backends (A via `IGeometryBackend`); History (A via `ITransactionObserver`).
**Provides:** `IEditingService` implementation (E-11), E-04 lifecycle, backend registry.

**Primary responsibility:** Transactional Editing API and coordination.

## Objective

Implement the orchestration layer connecting public editing commands to backend operations.

## Scope

- Transaction lifecycle.
- Command validation.
- Operation dispatch.
- Atomic commit and rollback.
- Revision checks.
- Dirty Tracking integration.
- Dependency Graph notifications.
- Typed error propagation.
- Cancellation handling.
- Backend selection.

## Required Interfaces

Conceptual contract:

```
EditingOrchestrator
  BeginTransaction(context)
  Execute(command)
  CommitTransaction()
  RollbackTransaction()
  CancelOperation(operationId)
  GetOperationStatus(operationId)
```

Exact signatures must be approved before implementation.

## Must Not Implement

- Voxel algorithms.
- SDF algorithms.
- Mesh topology algorithms.
- History storage internals.
- Persistence serialization.

## Tests

- Successful transaction.
- Failed transaction rollback.
- Revision conflict.
- Invalid Layer.
- Unsupported operation.
- Cancellation.
- Concurrent independent region edits.
- Dirty Tracking propagation.
- Async result invalidation.

## Acceptance

- No partial authoritative mutation on failed atomic transactions.
- Correct dispatch to specialized backends.
- Revision validation enforced.
- Dirty Tracking cannot be bypassed.
- Public errors are typed.
- Tests and documentation complete.

# 4. Agent B — Voxel Backend

**Depends on:** Core, Processing, Editing API (D). Does not depend on Orchestration or Tools.
**Provides:** `IGeometryBackend` for CubicVoxel.

**Primary responsibility:** Cubic Voxel Editing Backend.

## Objective

Implement native geometry editing for cubic voxel layers.

## Scope

- Voxel read/write.
- Occupancy changes.
- Material assignment.
- Attribute channel modification.
- Region-based operations.
- Local voxel addressing.
- Brick boundary handling.
- Sparse and dense storage compatibility.
- Brick-level validation.

## Operations

- SetVoxel.
- ClearVoxel.
- SetMaterial.
- FillRegion.
- RemoveRegion.
- CopyRegion.
- TransformRegion where supported.

## Constraints

- Base Brick: 16³.
- Adaptive channel storage.
- Signed Brick coordinates.
- Correct negative-coordinate addressing.
- No implicit SDF or Mesh conversion.
- No assumptions that all Bricks are dense.

## Tests

- Boundary coordinates.
- Negative coordinates.
- Empty Brick.
- Missing Brick.
- Sparse storage.
- Dense storage.
- Constant channels.
- Material palette boundaries.
- Large region modifications.
- Concurrent independent regions.

## Acceptance

- Correct addressing.
- No data corruption at Brick boundaries.
- Correct channel behavior.
- Revision and Dirty Tracking integration.
- Benchmark results delivered.

# 5. Agent C — SDF Backend

**Depends on:** Core, Processing, Editing API (D). Does not depend on Orchestration or Tools.
**Provides:** `IGeometryBackend` for SDF.

**Primary responsibility:** SDF Geometry Editing Backend.

## Objective

Implement native editing of signed distance fields.

## Scope

- SDF sampling.
- Density modification.
- Local field operations.
- Surface-preserving modifications.
- Gradient and normal support.
- Quantized field compatibility.
- Refinement Region integration.

## SDF Convention

- Negative: inside.
- Zero: surface.
- Positive: outside.

## Constraints

- Material IDs must not be interpolated numerically.
- Field values and categorical channels require different handling.
- Preserve source precision where supported.
- Avoid unnecessary full-volume recomputation.
- Respect refinement boundaries.

## Tests

- Sign convention.
- Surface extraction compatibility.
- Gradient behavior.
- Quantization error.
- Boundary continuity.
- Smooth operations.
- Local modifications.
- Resampling interaction.
- Revision invalidation.

## Acceptance

- Correct field semantics.
- No unexplained sign inversion.
- Stable surface behavior.
- Localized updates where possible.
- Performance and precision reports delivered.

# 6. Agent D — Mesh Backend

**Depends on:** Core, Processing, Editing API (D). Does not depend on Orchestration or Tools.
**Provides:** `IGeometryBackend` for PolygonMesh.

**Primary responsibility:** Polygon Mesh Editing Backend.

## Objective

Implement native editing of Polygon Mesh Layers.

## Scope

- Vertex operations.
- Edge operations.
- Face operations.
- Topology modification.
- Mesh validation.
- Normal recalculation.
- Attribute updates.
- Local mesh region identification.
- Mesh revision tracking.

## Constraints

- Mesh may be a derived representation or an imported authoritative Mesh Layer.
- These cases must remain distinct.
- Topology-changing operations must invalidate dependent data.
- Do not assume manifold topology.
- Invalid geometry must produce explicit validation results.

## Tests

- Vertex movement.
- Face modification.
- Topology changes.
- Degenerate geometry.
- Non-manifold cases.
- Attribute preservation.
- Normal updates.
- Bounds updates.
- Failed operation rollback.

## Acceptance

- Correct topology handling.
- Explicit validation.
- No accidental conversion of derived mesh into authoritative source.
- Dependency invalidation.
- Performance reports.

# 7. Agent E — Sculpting Tools

**Depends on:** Core, Processing, Editing API (D); Orchestration (A via `IEditingService`); Backends (A via capabilities). **Does not depend on History.**

**Primary responsibility:** Unified Sculpting Tool Layer.

## Objective

Implement a common sculpting interface with specialized backend dispatch.

## Tools

- Add.
- Remove.
- Smooth.
- Grab.
- Move.
- Draw.
- Crease.
- Flatten.
- Inflate.

## Brush Parameters

```
BrushParameters
  Radius
  Strength
  Falloff
  Hardness
  Spacing
  Symmetry
```

## Scope

- Common tool interface.
- Brush configuration.
- Input sampling.
- Stroke lifecycle.
- Backend dispatch.
- Region calculation.
- Dirty Region reporting.
- Transaction integration.
- Undo integration.
- Optional modifier-based sculpting.

## Backend Rules

- Voxel operations modify occupancy and attributes.
- SDF operations modify the scalar field.
- Mesh operations modify vertices or topology.

The common API must not pretend these algorithms are identical.

## Tests

- Each tool per supported backend.
- Brush boundary behavior.
- Falloff.
- Strength.
- Symmetry.
- Stroke cancellation.
- Undo.
- Large strokes.
- Refinement boundaries.

## Acceptance

- Consistent public behavior.
- Backend-specific correctness.
- No hidden geometry conversion.
- Transaction integration.
- Interactive performance measurements.

# 8. Agent F — Boolean Operations

**Depends on:** Core, Processing, Editing API (D); Orchestration (A); Backends (A via capabilities). **Does not depend on History.**
**Provides:** `BooleanPolicy` (not yet written, next contract pass).

**Primary responsibility:** Unified Boolean Editing System.

## Objective

Implement explicit Boolean operations across supported geometry representations.

## Operations

- Union.
- Subtract.
- Intersect.
- Split.

## Scope

- Unified public dispatch.
- Native same-type backend selection.
- Explicit mixed-type conversion policies.
- Operand validation.
- Result creation.
- Material mapping.
- Attribute policy.
- Resolution policy.
- Async execution.
- Revision validation.
- Cancellation.
- Transaction integration.

## Important Distinction

Surface Union is derived visual composition.

Boolean Merge modifies or creates geometry data.

Do not combine these semantics.

## Tests

- Same-type operations.
- Mixed-type rejection without conversion policy.
- Explicit conversion.
- Empty operands.
- Invalid operands.
- Degenerate cases.
- Large operands.
- Cancellation.
- Revision conflict.
- Rollback.
- Material preservation.
- Repeated operations.

## Acceptance

- No implicit conversion.
- Typed failure behavior.
- Correct result ownership.
- Async revision safety.
- Undo-compatible output.
- Benchmark and robustness report.

# 9. Agent G — History System

**Depends on:** Core, Processing, Editing API (D); Orchestration (A via `IEditingService` for undo); Persistence (A via `IDeltaStore`/`ISnapshotStore`).
**Rule:** History owns no Brick Storage and never imports Persistence internals.

**Primary responsibility:** Undo / Redo and Edit History.

## Objective

Implement the hybrid Commands + Deltas + Snapshots architecture on top of Core copy-on-write versions and the `IDeltaStore`/`ISnapshotStore` ports; implement no Brick Storage of your own.

## Scope

- Command recording.
- Delta representation.
- Snapshot creation.
- Checkpoints.
- Undo.
- Redo.
- History limits.
- Local history.
- Server-confirmed history integration.
- Memory accounting.
- Recovery support.

## Rules

Commands preserve operation intent.

Deltas preserve actual changes.

Snapshots accelerate restoration.

Heavy operations must not depend exclusively on replaying expensive algorithms.

## Tests

- Commit and Undo.
- Redo.
- Nested or grouped transactions where supported.
- Failed transaction.
- Snapshot restore.
- Delta restore.
- History truncation.
- Memory limits.
- Revision conflict.
- Server-authorized changes.
- Large operations.

## Acceptance

- Exact restoration within the representation's defined semantics.
- Correct dependency invalidation.
- No unbounded history memory.
- Local and server histories remain distinct.
- Persistence compatibility.

# 10. Agent H — Persistence Integration

**Depends on:** Core, Processing, Editing API (D). Does not depend on History, Orchestration or Backends; does not control editing.
**Provides:** `IDeltaStore`, `ISnapshotStore`, P-01…P-07.

**Primary responsibility:** Geometry Editing persistence integration.

## Objective

Implement the serialized format, durable stores and recovery for Geometry Editing, using the Geometry Core runtime storage through its export/import views.

## Scope

- Asset Manifest updates.
- Brick persistence integration.
- Incremental Delta storage.
- Snapshot checkpoints.
- Atomic save.
- Recovery.
- Version compatibility.
- Content Hash integration.
- Deduplication.
- Composite references.
- Async persistence.

## Constraints

- Do not invent a second authoritative runtime storage system; Core owns runtime storage.
- Persistence owns the serialized format; Core provides only export/import views (C-08).
- Do not import History or Orchestration internals.
- Manifest must not reference unavailable Brick data.
- Preserve unknown optional channels where possible.
- No blocking main-thread save operations.

## Tests

- Save and reload.
- Partial save interruption.
- Manifest consistency.
- Missing Brick detection.
- Delta replay.
- Snapshot recovery.
- Migration.
- Corrupt data.
- Content Hash validation.
- Composite reference restoration.

## Acceptance

- Recoverable persistence.
- Atomic commit behavior.
- Versioned format compatibility.
- No lost committed edits.
- Async save behavior verified.

# 11. Agent I — Tests and Benchmarks

**Depends on:** all modules, test-time only. Never a runtime dependency.

**Primary responsibility:** QA infrastructure and cross-module validation.

## Objective

Create the test and benchmark system for Geometry Editing.

## Scope

- Unit test infrastructure.
- API contract tests.
- Integration tests.
- Property-based tests.
- Regression tests.
- Determinism tests.
- Stress tests.
- Persistence tests.
- Multiplayer tests.
- Performance benchmark harness.
- CI integration.

## Required Benchmark Categories

1. Brick editing.
2. Sparse versus dense storage.
3. SDF modification.
4. Mesh editing.
5. Sculpting.
6. Boolean operations.
7. Resolution refinement.
8. Undo / Redo.
9. Snapshot creation.
10. Persistence.
11. Async scheduling.
12. Concurrent region editing.

## Test Data

Create reproducible datasets:

- Small Asset.
- Medium Asset.
- Large Asset.
- Highly sparse Asset.
- Dense Asset.
- High-detail Asset.
- Multi-Layer Asset.
- Composite Asset.
- Mixed-resolution Asset.

## Acceptance

- Reproducible tests.
- Machine-readable benchmark reports.
- CI failure on required test failures.
- Regression tracking.
- Performance baselines.
- No unsupported claims of production readiness.

# 12. Shared Integration Rules

All agents:

- Use approved API contracts.
- Own only assigned modules.
- Submit separate PRs.
- Include tests and documentation.
- Report dependencies.
- Report deviations.
- Do not change shared contracts unilaterally.

# 13. Initial Execution Order

## Phase 0 — Repository & Decision Readiness

- Integration Owner records language/engine decisions (AU-1.0 §24 #1–2) and sets up repository layout, CI skeleton and the dependency-direction check.
- Missing source documents are added to the repository (GDD/TDD, Voxel Architecture, Geometry Processing).
- Owners for Networking Adapter and Engine Adapters are assigned (or explicitly deferred).

## Phase 1 — Shared Contracts

Integration Owner approves contracts by tier (`docs/api-contracts/README.md` §3): **P0** (C-01…C-19) before Core starts; **P1** (PR-01…PR-08, E-02…E-07, E-09, E-11, E-12) before Processing, Orchestration and backends start; **P2** before History, Persistence and modifier work start. Agent I starts test infrastructure.

## Phase 2 — Foundations (sequential gate)

1. Core Agent implements Geometry Core.
2. Processing Agent implements Processing foundations (contracts may be developed against the approved Core API while Core is being implemented, merge after Core exit).
3. Integration Owner finalises Editing API & Contracts.

Agent H may start once Core, Processing and Editing contracts are approved (no dependency on Orchestration or History).

## Phase 3 — Dispatch & Backends

- Agent A: Orchestration.
- Agents B, C, D: backends against the conformance suite.

## Phase 4 — Editing Features

- Agent E Sculpting, Agent F Boolean, Resolution Agent, Agent G History, Composite Agent — parallel, each starting when Orchestration and at least one usable backend are available.

## Phase 5 — Boundaries

- Networking Adapter and Engine Integration Adapters (owners pending).

## Phase 6 — Full Integration & Acceptance

All modules pass layered integration and cross-module tests; Integration Owner evaluates the acceptance checklist.

# 14. Agent Delivery Package

Every agent must deliver:

1. Source code.
2. Tests.
3. API documentation.
4. Dependency declaration.
5. Benchmark report where applicable.
6. Known limitations.
7. Integration instructions.
8. Completion report.

**End of Agent Task Pack.**
