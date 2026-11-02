# Cube — Geometry Editing Agent Task Pack v1.0

**Document type:** Agent Work Assignment Package\
**Scope:** Geometry Editing v1.0\
**Coordination:** Central Integration\
**Contract authority:** Integration Owner

---

# 1. General Instructions

All agents must read:

- GDD/TDD v0.3.
- Voxel Architecture v1.0.
- Geometry Processing decisions.
- Geometry Editing Specification v1.0.
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

# 2. Task Dependency Overview

```
Geometry Core
      |
      v
Shared API Contracts
      |
      v
Editing Orchestration
      |
      +-----------------------------+
      |              |              |
      v              v              v
Voxel Backend    SDF Backend    Mesh Backend
      |              |              |
      +--------------+--------------+
                     |
                     v
              Sculpting Tools
                     |
                     v
              Boolean Operations
                     |
                     v
               History System
                     |
          +----------+----------+
          |                     |
          v                     v
  Composite Geometry    Persistence Integration
          |                     |
          +----------+----------+
                     |
                     v
              Integration & QA
```

This diagram shows logical dependencies, not permission to skip the layered build pipeline. Actual task execution may overlap after contracts and required dependencies are stable.

# 3. Agent A — Editing Orchestration

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

**Primary responsibility:** Undo / Redo and Edit History.

## Objective

Implement the hybrid Commands + Deltas + Snapshots architecture.

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

**Primary responsibility:** Geometry Editing persistence integration.

## Objective

Connect editing operations with the existing Geometry Core storage and serialization systems.

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

- Do not invent a second authoritative storage system.
- Reuse approved Geometry Core serialization interfaces.
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

## Phase 0 — Repository Inspection

All agents inspect repository structure, existing conventions and available Core interfaces.

No overlapping implementation before ownership and contracts are agreed.

## Phase 1 — Shared Contracts

Integration Owner defines and approves:

- Asset and Layer handles.
- Revision model.
- Dirty Region model.
- Transaction contract.
- Backend dispatch.
- Async job contract.
- Typed errors.
- History integration.
- Persistence integration.

## Phase 2 — Core Dependencies

- Agent A: Orchestration skeleton.
- Agents B, C, D: backend foundations.
- Agent I: test infrastructure.

## Phase 3 — Editing Features

- Agent E: Sculpting.
- Agent F: Boolean.
- Agent G: History.
- Agent H: Persistence Integration.

## Phase 4 — Full Integration

All modules pass layered integration and cross-module tests.

## Phase 5 — Acceptance

Integration Owner evaluates the full acceptance checklist.

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
