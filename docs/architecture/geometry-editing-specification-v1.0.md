# Cube — Geometry Editing Specification v1.0

**Document type:** Technical Architecture Specification\
**Status:** Architecture approved; implementation specification\
**Project:** Cube — Voxel Open World RPG / MMORPG\
**Related documents:**

- GDD/TDD v0.3
- Voxel Architecture v1.0
- Geometry Processing Specification (согласованные решения)
- Cube Agent Development Specification v1.0

---

# 1. Purpose and Scope

Geometry Editing — универсальная система создания, изменения, объединения, скульптинга, сохранения и синхронизации геометрии.

Система должна поддерживать:

- Cubic Voxel Geometry.
- SDF / Density Geometry.
- Polygon Mesh Geometry.
- Composite Geometry.
- Hybrid Geometry Assets.
- Local Resolution Refinement.
- Destructive и Non-Destructive Editing.
- Sculpting.
- Boolean Operations.
- Undo / Redo.
- Multiplayer Editing.
- Incremental Persistence.

Система предназначена для игрового мира, строительных объектов, оружия, брони, статуй, окружения и сложных составных моделей.

## 1.1 Design Principles

1. Engine-independent Geometry Core.
2. Unified Public Editing API.
3. Specialized Geometry Backends.
4. Strict Module Contracts.
5. Transactional Editing.
6. Region-Based Change Tracking.
7. Asynchronous Processing.
8. Revision Validation.
9. Explicit Geometry Conversion.
10. Deterministic and Versioned Persistence.
11. Mobile-aware Performance.
12. Server-authoritative Multiplayer.

# 2. Architecture Overview

## 2.1 Module Structure

```
Geometry Editing API
        |
        v
Editing Orchestration
        |
        +-- Editing Backends
        |     +-- Voxel Backend
        |     +-- SDF Backend
        |     +-- Mesh Backend
        |
        +-- Editing Tools
        |     +-- Sculpting
        |     +-- Boolean
        |     +-- Resampling
        |
        +-- History System
        |     +-- Commands
        |     +-- Deltas
        |     +-- Snapshots
        |
        +-- Composite Geometry
        |     +-- Hierarchy
        |     +-- Assembly Graph
        |
        +-- Processing
        |     +-- Dependency Graph
        |     +-- Async Jobs
        |     +-- Revision Validation
        |
        +-- Persistence Integration
              +-- Manifest
              +-- Bricks
              +-- Deltas
              +-- Snapshots

Geometry Core
```

Geometry Core owns the fundamental geometry data structures. Editing modules must access them through approved interfaces.

## 2.2 Architectural Ownership

| Module        | Responsibility                     |
| ------------- | ---------------------------------- |
| Geometry Core | Assets, Layers, Bricks, channels   |
| Editing API   | Public editing interface           |
| Orchestration | Commands, transactions, validation |
| Voxel Backend | Cubic voxel operations             |
| SDF Backend   | Scalar field operations            |
| Mesh Backend  | Polygon mesh operations            |
| Sculpting     | Brush tools and sculpt operations  |
| Boolean       | Geometry Boolean operations        |
| Resampling    | Resolution changes                 |
| History       | Undo, Redo, checkpoints            |
| Composite     | Hierarchy and assembly             |
| Processing    | Dependencies and async jobs        |
| Persistence   | Save/load integration              |

# 3. Geometry Editing Data Model

## 3.1 Geometry Asset

Geometry Asset is the primary editable resource.

Conceptual structure:

```
GeometryAsset
  AssetId
  AssetRevision
  Metadata
  Bounds
  GeometryLayers[]
  Junctions[]
  CompositeReferences[]
  DependencyMetadata
```

Asset is not required to store all geometry in one representation.

## 3.2 Geometry Layer

Each Layer owns one geometry representation and its own configuration.

```
GeometryLayer
  LayerId
  AssetId
  GeometryType
  LayerRevision
  Transform
  ResolutionProfile
  MaterialPalette
  AttributeChannels
  GeometryStorage
  Modifiers[]
  CollisionProfile
  LayerMetadata
```

GeometryType:

- CubicVoxel
- SDF
- PolygonMesh

A Layer has one authoritative source representation. A derived mesh or collision structure is not automatically authoritative.

## 3.3 Geometry Layers

Layers remain independent by default.

Allowed relationships:

- Separate.
- Blend.
- Union.
- Boolean.

No implicit merge is allowed.

## 3.4 Attribute Channels

Attributes are extensible and representation-aware.

Possible channels:

- Occupancy.
- Material ID.
- Density.
- SDF Value.
- Surface Normal.
- Color.
- UV.
- Emission.
- Roughness.
- Custom gameplay attributes.

Unsupported channels must be preserved when possible during serialization and migration.

## 3.5 Material System

Each Geometry Asset uses a local material palette.

Material IDs reference the global material library.

Material IDs must not be treated as interpolatable numeric values.

# 4. Geometry Storage

## 4.1 Brick Architecture

Base Brick size: **16³**.

BrickKey:

```
BrickKey {
  LayerId
  BrickCoord
}
```

BrickCoord uses signed integer coordinates.

Local addressing:

```
Index = x + B * (y + B * z)
B = 16
```

This indexing order is provisional and must be benchmarked.

## 4.2 Storage Modes

Channel storage is adaptive:

- Dense.
- Sparse.
- Constant.
- Bitset where appropriate.

Serialized Storage and Runtime Storage are separate systems.

## 4.3 Compression

Candidate methods:

- Run-Length Encoding.
- Bit Packing.
- Dictionary Encoding.
- LZ4.
- Zstandard.

Final combinations must be selected through benchmarks.

## 4.4 Missing Data States

A missing Brick must distinguish:

- Known Empty.
- Not Loaded.
- Outside Asset Bounds.
- Load Failure.

These states must never be conflated.

# 5. Editing API Architecture

## 5.1 Hybrid Editing API

Two levels:

**Low-Level Editing API**

- Direct data modifications.
- Region-based changes.
- Backend-specific operations.
- Dirty Tracking.
- Revision updates.

**Transactional Editing API**

- Commands.
- Validation.
- Atomic execution.
- Undo / Redo.
- History.
- Network synchronization.

Low-level operations cannot bypass invalidation, Revision management or dependency tracking.

## 5.2 Unified Public API

All geometry types use one public editing interface.

Backend implementations remain specialized.

Examples of conceptual operations:

```
BeginTransaction()
CommitTransaction()
RollbackTransaction()

SetMaterial()
AddGeometry()
RemoveGeometry()
TransformGeometry()

BooleanUnion()
BooleanSubtract()
BooleanIntersect()
BooleanSplit()

BeginSculpt()
ApplyBrush()
EndSculpt()

ResampleLayer()
BakeModifiers()
CreateSnapshot()
```

These are conceptual operations, not final language-specific signatures.

## 5.3 Typed Errors

Required error categories:

- InvalidAsset.
- InvalidLayer.
- UnsupportedGeometryOperation.
- IncompatibleGeometryTypes.
- ConversionRequired.
- InvalidResolution.
- OutOfBounds.
- RevisionConflict.
- TransactionConflict.
- PermissionDenied.
- ResourceUnavailable.
- ProcessingCancelled.
- ProcessingFailed.
- PersistenceFailure.

Errors must carry actionable context without exposing unsafe internal memory references.

# 6. Editing Model

## 6.1 Destructive Editing

Directly modifies authoritative geometry data.

Suitable for:

- Runtime world modification.
- Building.
- Terrain destruction.
- Sculpting.
- Confirmed gameplay operations.

## 6.2 Non-Destructive Editing

Stores modifiers and parameters separately.

```
ModifierStack
  SourceGeometry
  Modifier[]
  EvaluatedGeometry
  EvaluationRevision
```

## 6.3 Bake

Bake can:

- Apply modifiers to the existing source Layer.
- Create a new Layer.
- Preserve the original source.

Bake must be transactional and undoable.

## 6.4 Evaluation

Evaluated Geometry is derived data.

It must be cached and invalidated through dependency tracking.

Not all operations are required to support Non-Destructive mode.

# 7. Sculpting System

## 7.1 Public Tool Interface

Common tools:

- Add.
- Remove.
- Smooth.
- Grab.
- Move.
- Draw.
- Crease.
- Flatten.
- Inflate.

## 7.2 Brush Parameters

```
BrushParameters
  Radius
  Strength
  Falloff
  Hardness
  Spacing
  Symmetry
```

Backend implementations differ:

- Voxel: occupancy and attributes.
- SDF: scalar field modification.
- Mesh: vertex and topology modification.

## 7.3 Requirements

- Localized editing.
- Region-based Dirty Tracking.
- Async evaluation where appropriate.
- Destructive and optional Non-Destructive execution.
- Undo / Redo.
- Resolution-aware behavior.
- No implicit conversion between geometry types.

# 8. Resolution Architecture

## 8.1 Per-Layer Resolution

Each Layer has an independent base resolution.

## 8.2 Local Resolution Refinement

Refinement Regions and Refinement Bricks allow localized detail.

The entire Asset must not require higher resolution when only a small region needs refinement.

## 8.3 Requirements

- Common spatial coordinate system.
- Hierarchical refinement levels.
- Correct boundary transitions.
- No visible cracks between resolutions.
- Resampling must preserve shape, materials and supported attributes as far as possible.
- Affected dependencies must be invalidated.
- Refinement and resampling must support Undo.

Exact resolution profiles are benchmark-driven. Candidate profiles include 8³, 16³, 32³, 64³ and 100³, not mandatory universal limits.

# 9. Boolean Architecture

## 9.1 Operations

- Union.
- Subtract.
- Intersect.
- Split.

## 9.2 Backend Rules

Same-type operands use native backend implementations where available.

Mixed-type operands require an explicit conversion policy.

No implicit conversion is permitted.

## 9.3 Output

Boolean result may:

- Create a new Layer.
- Modify an existing Layer.

The operation must define:

- Source operands.
- Target.
- Conversion policy.
- Resolution policy.
- Material policy.
- Attribute policy.
- Failure behavior.

## 9.4 Surface Union vs Boolean Merge

Surface Union is a derived visual representation.

Boolean Merge modifies or creates geometry data.

They are different operations and must not share ambiguous semantics.

## 9.5 Execution

Heavy Boolean operations:

- Run asynchronously.
- Use immutable snapshots.
- Validate source revisions before commit.
- Support cancellation.
- Use temporary region locks where needed.
- Produce transactional results.

# 10. Geometry Junctions

Explicit Junction modes:

- Separate.
- Blend.
- Union.
- Boolean.

Junction data conceptually contains:

```
GeometryJunction
  LayerA
  LayerB
  Mode
  Region
  Parameters
  Revision
```

Automatic geometry merging is forbidden.

# 11. Composite Geometry

## 11.1 Hybrid Hierarchy and Assembly Graph

Hierarchy:

- Parent / Child.
- Local transforms.
- Transform propagation.

Assembly Graph:

- Attachment Points.
- Constraints.
- Dependencies.
- Part relationships.

## 11.2 Part Data

Each part may own:

- Geometry Asset reference.
- Transform.
- Durability.
- Metadata.
- Collision Profile.
- Attachment Points.
- Gameplay state.

## 11.3 Operations

- Attach.
- Detach.
- Reparent.
- Transform.
- Apply Constraints.
- Destroy Part.

Composite Geometry does not merge geometry automatically.

# 12. History System

## 12.1 Hybrid History Model

History combines:

- Commands.
- Deltas.
- Copy-on-Write Snapshots.
- Checkpoints.

Commands store operation intent and parameters.

Deltas store actual data changes.

Snapshots accelerate restoration.

## 12.2 Undo / Redo

Undo and Redo must:

- Preserve data consistency.
- Restore revisions correctly.
- Invalidate dependent caches.
- Integrate with transactions.
- Respect authorization.

Heavy operations may store result deltas or snapshots rather than relying exclusively on algorithm replay.

## 12.3 History Limits

Limits are configurable:

- Memory.
- Number of operations.
- Retention time.
- Snapshot frequency.

Local history is separate from server history.

# 13. Multiplayer Concurrency

## 13.1 Hybrid Region-Based Concurrency

Independent regions may be edited concurrently.

Revision levels:

- Asset.
- Layer.
- Region.

## 13.2 Server Authority

The client may show optimistic previews.

The server remains authoritative.

## 13.3 Conflict Detection

Conflicts are checked against actual read/write regions and dependencies.

## 13.4 Heavy Operations

Boolean and Resolution Refinement may use temporary locks.

## 13.5 Reconciliation

After server response:

- Confirm accepted operations.
- Reject invalid operations.
- Reconcile client previews.
- Update revisions.
- Record confirmed changes.

Unfinished client operations after reconnect require server validation before replay.

# 14. Persistence Integration

## 14.1 Storage Architecture

Hybrid architecture:

```
Asset Manifest
  Metadata
  Layers
  Resolution Profiles
  Material Palettes
  Dependencies

Brick Storage
  LayerId
  Brick Coordinates
  Serialized Data

Incremental Deltas
  Changes since Snapshot

Snapshots
  Checkpoints
```

## 14.2 Requirements

- Versioned format.
- Migrations.
- Integrity checks.
- Content Hash.
- Deduplication.
- Atomic Save.
- Recovery after interruption.
- Async persistence.
- Runtime / Serialized Storage separation.

## 14.3 Atomicity

Manifest must never reference missing Brick data after an interrupted save.

Related Asset parts require a consistent commit protocol.

Composite structure and references must survive reload.

# 15. Processing Architecture

## 15.1 Dependency Graph

Changes propagate through a dependency graph.

Possible nodes:

- Source Geometry.
- Evaluated Geometry.
- Surface Mesh.
- Collision.
- Hitboxes.
- Bounds.
- LOD.
- Persistence.

## 15.2 Immutable Snapshots

Async jobs consume immutable geometry snapshots.

Snapshot includes:

- Asset ID.
- Layer ID.
- Source revision.
- Brick snapshots.
- Dependency revisions.

## 15.3 Revision Validation

Before publishing a result, compare source and dependency revisions.

Stale results must be discarded.

## 15.4 Scheduling

Priority Queue:

- Visibility.
- Distance.
- Gameplay importance.
- User interaction.
- Background maintenance.

Starvation prevention is mandatory.

# 16. Module Architecture

Layered Modular Architecture.

Modules:

1. Geometry Editing API.
2. Editing Orchestration.
3. Editing Backends.
4. Editing Tools.
5. History System.
6. Composite Geometry.
7. Processing Integration.
8. Persistence Integration.
9. Geometry Core.

Each module exposes a strict public API.

No undocumented internal dependencies.

# 17. API Contract Requirements

All public contracts define:

- Types.
- Inputs and outputs.
- Ownership and lifetime.
- Thread-safety.
- Error behavior.
- Revision semantics.
- Dirty Tracking behavior.
- Async completion and cancellation.
- Compatibility policy.

Public API changes require central approval.

# 18. Integration Rules

Central Integration.

One Integration Owner controls:

- Public API changes.
- Dependency validation.
- Integration PRs.
- Build compatibility.
- Merge approval.
- Final release validation.

Agents develop modules independently against approved contracts.

# 19. Testing Architecture

Comprehensive Layered Testing:

1. Unit Tests.
2. Module Tests.
3. Integration Tests.
4. Property-Based Tests.
5. Regression Tests.
6. Performance Benchmarks.
7. Stress Tests.
8. Determinism Tests.
9. Persistence Tests.
10. Multiplayer Tests.

Every module must provide tests.

Critical operations require edge-case coverage.

# 20. Performance Requirements

Performance Budgets are mandatory.

Metrics:

- CPU Time.
- Memory Allocation.
- Peak Memory.
- Main-thread Time.
- Async Latency.
- Throughput.
- Scaling.

Numeric targets are established after prototype benchmarks on representative target hardware.

No arbitrary universal time or memory thresholds are introduced before measurement.

Performance regressions must be detected by CI.

# 21. Acceptance Criteria

Strict Acceptance Criteria.

A module is accepted only when:

1. Approved API Contract implemented.
2. Functional scope completed.
3. Unit Tests pass.
4. Module Tests pass.
5. Integration Tests pass.
6. Regression Tests pass.
7. Edge cases verified.
8. Performance requirements met or formally exempted.
9. No critical memory, concurrency or integrity defects.
10. Documentation updated.
11. Compatibility verified.
12. Integration Owner approves.

Compilation alone is not acceptance.

# 22. Final Integration Pipeline

Layered Build Pipeline:

1. Geometry Core.
2. Editing API and Contracts.
3. Editing Orchestration.
4. Voxel / SDF / Mesh Backends.
5. Sculpting / Boolean / Resampling.
6. History System.
7. Composite Geometry.
8. Processing Integration.
9. Persistence Integration.
10. Full Geometry Editing Integration.

Each stage must pass automated validation before the next stage.

No cyclic dependencies.

No bypassing architecture layers.

# 23. Agent Development Breakdown

Initial agent assignments:

| Agent | Responsibility          |
| ----- | ----------------------- |
| A     | Editing Orchestration   |
| B     | Voxel Backend           |
| C     | SDF Backend             |
| D     | Mesh Backend            |
| E     | Sculpting               |
| F     | Boolean                 |
| G     | History                 |
| H     | Persistence Integration |
| I     | Tests and Benchmarks    |

This is a proposed work breakdown, not permission to independently modify shared contracts.

Detailed tasks are defined in Geometry Editing Agent Task Pack v1.0.

# 24. Open Technical Decisions

The following require implementation evidence:

- Programming language and engine.
- Exact public API signatures.
- Exact numeric performance budgets.
- Brick compression combinations.
- Optimal storage layout and indexing order.
- SDF numeric format and precision.
- Mesh topology and validation implementation.
- Boolean algorithms for each backend.
- Refinement transition algorithms.
- Snapshot and delta encoding.
- CI infrastructure and benchmark hardware.

These decisions must be documented and approved before becoming normative implementation requirements.

# 25. Definition of Done

Geometry Editing v1.0 implementation is complete when:

- All required modules exist.
- Public contracts are implemented.
- All required geometry backends operate.
- Transactions and history work.
- Sculpting and Boolean operations work.
- Composite Geometry works.
- Async processing is revision-safe.
- Persistence is recoverable.
- Multiplayer concurrency is validated.
- Test suite passes.
- Performance budgets are established and met.
- Full integration is approved.

**End of Specification.**
