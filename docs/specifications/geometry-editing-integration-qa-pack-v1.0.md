# Cube — Geometry Editing Integration & QA Pack v1.0

**Document type:** Integration, Dependency and Quality Assurance Specification\
**Scope:** Geometry Editing v1.0\
**Owner:** Integration Owner

---

# 1. Module Dependency Matrix

Legend:

- Required: mandatory dependency.
- Optional: integration when available.
- None: no direct dependency.

| Module        | Core     | API      | Orchestration | Backends | Tools    | History  | Processing | Persistence |
| ------------- | -------- | -------- | ------------- | -------- | -------- | -------- | ---------- | ----------- |
| Orchestration | Required | Required | —             | Required | None     | Optional | Required   | None        |
| Voxel Backend | Required | Required | None          | —        | None     | None     | Required   | None        |
| SDF Backend   | Required | Required | None          | —        | None     | None     | Required   | None        |
| Mesh Backend  | Required | Required | None          | —        | None     | None     | Required   | None        |
| Sculpting     | Required | Required | Required      | Required | —        | Required | Required   | None        |
| Boolean       | Required | Required | Required      | Required | —        | Required | Required   | None        |
| Resampling    | Required | Required | Required      | Required | —        | Required | Required   | None        |
| History       | Required | Required | Required      | Optional | None     | —        | Required   | Required    |
| Composite     | Required | Required | Required      | Optional | None     | Optional | Required   | Required    |
| Persistence   | Required | Required | Optional      | None     | None     | Required | Required   | —           |
| QA            | Required | Required | Required      | Required | Required | Required | Required   | Required    |

This matrix describes logical integration dependencies. It does not authorize direct access to internal implementation details.

# 2. API Contract Matrix

All contracts must be approved before dependent modules implement against them.

| Contract                | Owner             | Consumers                        |
| ----------------------- | ----------------- | -------------------------------- |
| GeometryAssetHandle     | Geometry Core     | All modules                      |
| GeometryLayerHandle     | Geometry Core     | All modules                      |
| GeometryType            | Geometry Core     | API, Backends, Tools             |
| BrickAddress            | Geometry Core     | Voxel, Storage                   |
| AttributeChannel        | Geometry Core     | Backends, Tools                  |
| MaterialPalette         | Geometry Core     | Backends, Sculpting, Boolean     |
| RevisionToken           | Core / Processing | All editing modules              |
| DirtyRegion             | Core / Processing | Backends, Orchestration, Tools   |
| GeometrySnapshot        | Processing        | Async Backends, Boolean, History |
| EditingTransaction      | Orchestration     | Tools, Backends, History         |
| EditingCommand          | Orchestration     | History                          |
| GeometryOperationResult | Editing API       | All consumers                    |
| EditingError            | Editing API       | All modules                      |
| AsyncJobHandle          | Processing        | Orchestration, Tools             |
| CancellationToken       | Processing        | Async modules                    |
| ModifierDescriptor      | Editing API       | Sculpting, Evaluation            |
| BooleanPolicy           | Boolean           | Backends, Orchestration          |
| ResolutionProfile       | Geometry Core     | Backends, Resampling             |
| CompositeReference      | Composite         | Persistence, Orchestration       |
| PersistenceCommit       | Persistence       | History, Orchestration           |

# 3. Required API Contract Details

## 3.1 RevisionToken

Must identify the version of the relevant resource.

Required semantic fields:

- Asset identity.
- Layer identity where applicable.
- Region identity where applicable.
- Revision value.

Comparison must be explicit.

A revision token must not be treated as a global timestamp.

## 3.2 DirtyRegion

Conceptual structure:

```
DirtyRegion
  AssetId
  LayerId
  MinBounds
  MaxBounds
  ChangeType
  SourceRevision
```

ChangeType:

- Geometry.
- Material.
- Attribute.
- Transform.
- Topology.
- Resolution.

Dirty regions represent actual changed data.

Processing may expand its working region according to algorithm dependencies without incorrectly marking the expanded area as modified.

## 3.3 GeometrySnapshot

Must include:

- Asset ID.
- Layer ID.
- Source revision.
- Required immutable Brick references.
- Dependency revisions.
- Snapshot lifetime information.

A worker must not mutate snapshot data.

## 3.4 EditingTransaction

Must define:

- Transaction identity.
- Target resources.
- Read and write regions.
- Expected revisions.
- Operation sequence.
- Commit policy.
- Rollback policy.
- Authorization context.

## 3.5 AsyncJobHandle

Must expose:

- Job identity.
- Status.
- Cancellation.
- Result availability.
- Failure information.
- Revision dependencies.

Possible states:

```
Created
Queued
Running
Completed
Failed
Cancelled
Stale
```

## 3.6 GeometryOperationResult

Must distinguish:

- Success.
- Validation failure.
- Conflict.
- Unsupported operation.
- Conversion required.
- Resource failure.
- Cancellation.

Partial results require an explicit contract.

# 4. Module Ownership Matrix

| Module           | Primary Owner     | Forbidden Responsibilities                         |
| ---------------- | ----------------- | -------------------------------------------------- |
| Orchestration    | Agent A           | Backend algorithms                                 |
| Voxel            | Agent B           | SDF/Mesh implementation                            |
| SDF              | Agent C           | Voxel/Mesh implementation                          |
| Mesh             | Agent D           | Voxel/SDF implementation                           |
| Sculpting        | Agent E           | Owning backend storage                             |
| Boolean          | Agent F           | Owning global geometry storage                     |
| History          | Agent G           | Directly bypassing transactions                    |
| Persistence      | Agent H           | Replacing Geometry Core storage                    |
| QA               | Agent I           | Changing production behavior to hide test failures |
| Shared Contracts | Integration Owner | Unreviewed changes                                 |

# 5. Layered Build Pipeline

## Stage 1 — Geometry Core

Validation:

- Asset model.
- Layer model.
- Brick addressing.
- Channel storage.
- Material palette.
- Revision primitives.
- Serialization primitives.

Exit condition: Core tests pass.

## Stage 2 — Editing API and Contracts

Validation:

- Public type definitions.
- Error model.
- Ownership.
- Lifetime.
- Threading.
- Revision.
- Async semantics.

Exit condition: Contract tests pass.

## Stage 3 — Editing Orchestration

Validation:

- Transaction lifecycle.
- Backend dispatch.
- Rollback.
- Dirty Tracking.
- Revision validation.

Exit condition: Orchestration integration tests pass.

## Stage 4 — Geometry Backends

Validation:

- Voxel backend.
- SDF backend.
- Mesh backend.
- Common public API compliance.
- Backend-specific correctness.

Exit condition: All required backend tests pass.

## Stage 5 — Editing Tools

Validation:

- Sculpting.
- Boolean.
- Resampling.
- Explicit conversion.
- Transaction integration.

Exit condition: Tool integration tests pass.

## Stage 6 — History

Validation:

- Commands.
- Deltas.
- Snapshots.
- Undo.
- Redo.
- Memory limits.

Exit condition: History restoration tests pass.

## Stage 7 — Composite Geometry

Validation:

- Hierarchy.
- Assembly Graph.
- Attach / Detach.
- Constraints.
- Transform propagation.

Exit condition: Composite integration tests pass.

## Stage 8 — Processing Integration

Validation:

- Dependency Graph.
- Async Jobs.
- Snapshot consistency.
- Revision validation.
- Stale result rejection.
- Scheduling.

Exit condition: Concurrency and async tests pass.

## Stage 9 — Persistence Integration

Validation:

- Manifest.
- Brick storage.
- Delta storage.
- Snapshots.
- Atomic save.
- Recovery.
- Migration.

Exit condition: Persistence and recovery tests pass.

## Stage 10 — Full Integration

Validation:

- Complete dependency graph.
- All public contracts.
- Full test suite.
- Stress tests.
- Performance baselines.
- Regression suite.

Exit condition: Integration Owner approval.

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

The following remain implementation-dependent:

- Numeric performance thresholds.
- Exact engine and language.
- Exact API signatures.
- Exact Boolean algorithms.
- Exact SDF storage precision.
- Refinement transition algorithms.
- Benchmark infrastructure.
- CI provider.
- Final serialization format.

These must be resolved through implementation evidence and approved technical decisions.

**End of Integration & QA Pack.**
