# Cube — Geometry Processing Contracts

**Status:** DRAFT 0.1 — specified for approval; none approved.
**Implementing module:** Processing (Processing Agent). Basis: AU-1.0 §5.3, §9–§12, §23.
Conventions: `README.md`. Field order: Purpose · Model · Ownership · Lifetime · Access · Revision · Dirty · Threading · Cancel · Errors · Serialization · Compat · Tests · Open.

---

## 0. Processing Architecture (place in the stack)

- Layer 1, directly above Geometry Core. Depends only on Core (read views, BrickStorage pins, Bounds, RevisionToken, DirtyRegion, Result/Error).
- Contains **no editing or game business logic**. It executes derived-data computations that others register.
- Components: Dependency Graph (static module deps + dynamic derived-data/task graph), Job Scheduler (priority queue, dependency-aware, starvation prevention, resource limits, main-thread budget, platform policies), Snapshot service, Revision validator, Invalidation engine, Derived-data caches (Bounds, BVH/spatial hierarchy, Surface Mesh slot, Collision slot, Hitbox slot, LOD slot).
- Processing defines **ports** for engine facilities (job executor, allocator, clock); engine adapters implement them (inversion, so Core/Processing stay engine-independent).
- Which algorithm builds Surface Mesh / Collision / LOD is supplied by **providers registered by their owners**: Surface Nets for SDF and greedy meshing for voxel are named in AU-1.0 §21 but have **no owner assigned** (Open Decision O-1 in the audit report).

### Processing responsibilities by concern

| Concern | Behaviour |
|---|---|
| Dependency Graph | nodes = derived-data slots (Evaluated Geometry, Surface Mesh, Collision, Hitbox, Bounds, BVH, LOD) and tasks; edges = "slot X consumes inputs Y"; cycles rejected at registration |
| Job Scheduler | priority from visibility, distance, gameplay importance, user interaction, background; aging for starvation; per-resource limits |
| Snapshot Input | every task runs on a `ProcessingSnapshot` only |
| Revision Validation | at publish; see semantics §4.2 |
| Dirty Region Expansion | `ProcessingRegion = DirtyRegion expanded by provider-declared radius`, clamped to bounds; never written back |
| Cache Invalidation | on commit notification: mark affected slots stale by `ChangeType` (Geometry → mesh/collision/hitbox/LOD/bounds; Material → mesh attributes only; Attribute → affected channels' consumers; Transform → world bounds/BVH/collision-world; Topology → all geometry-derived; Resolution → all) |
| Cancellation | cooperative via `CancellationToken`; cancelled task releases pins |
| Stale Rejection | result discarded; no publication |
| Bounds updates | task kind `BoundsUpdate`: world AABB, OBB, sphere |
| BVH updates | task kind `BvhUpdate`: incremental where possible |
| LOD | task kind `LodBuild`; scheduling by distance |

---

## PR-01 ProcessingSnapshot

- **Purpose:** immutable consistent input for async work.
- **Model:** `AssetId, LayerId, LayerToken, BrickPins[], DependencyRevisions[], OperationContext?, LifetimePolicy`. Capture protocol: semantics §3.
- **Ownership:** Processing owns snapshot object; Core owns pinned payload. **Lifetime:** ref-counted; released at task end; max age/memory Open (OD-R4).
- **Access:** read-only for workers; mutation is impossible by construction.
- **Revision:** carries tokens, compares at publish. **Dirty:** n/a (reads only).
- **Threading:** freely shareable across workers. **Cancel:** release on cancel. **Errors:** `ResourceUnavailable` if memory cap hit.
- **Serialization:** none (runtime only); persistence snapshots are P-04. **Compat:** n/a.
- **Tests:** immutability under concurrent commit; pin release on failure; MVP copy-vs-pin equivalence.
- **Open:** pin vs copy threshold.

## PR-02 DependencyRevision

- **Purpose:** exact input stamp of a derived result.
- **Model:** `SlotId, InputTokens[(RevisionToken)], ProviderId, ProviderVersion, ParametersHash`.
- **Ownership:** Processing. **Lifetime:** with the derived-data cache entry.
- **Revision rule:** derived value is current iff every input token still equals the current token (within ProcessingRegion).
- **Dirty:** drives invalidation edges. **Threading:** value-safe. **Serialization:** optional with cached derived data (not authoritative). **Tests:** provider version change invalidates; parameter change invalidates.
- **Open:** granularity of inputs (per-brick vs per-region stamp).

## PR-03 ProcessingTask

- **Purpose:** unit of async work (also the job handle exposed upward).
- **Model:** `TaskId, Kind (extensible), Priority, SnapshotRef, InputsRegion(ProcessingRegion), OperationId?, State, CancellationToken, DependsOn[], ResultSlot`.
- **State machine:** `Created → Queued → Running → {Completed | Failed | Cancelled | Stale}`; terminal states are final; `Completed` means published; `Stale` means finished but rejected.
- **Ownership:** Processing owns the task; submitter holds `AsyncJobHandle` (read: status, result availability, failure info, revision dependencies; write: Cancel).
- **Access:** handles are safe from any thread. **Revision:** records the DependencyRevision. **Dirty:** a Completed task may produce no dirty (derived data); operation-level tasks (Boolean) return a proposed transaction, not a mutation.
- **Cancel:** cooperative, promptly releases pins; Cancel after terminal state is a no-op.
- **Errors:** `ProcessingFailed`, `ProcessingCancelled`, `ResourceUnavailable`. **Serialization:** none. **Tests:** state-machine legality; starvation prevention; worker failure releases resources; cancellation race.
- **Open:** task kind extension mechanism; handle exposure to Tools (AsyncJobHandle in QA matrix maps to this).

## PR-04 ProcessingResult

- **Purpose:** typed outcome of a task before publication.
- **Model:** `TaskId, Outcome (Published | Stale | Failed | Cancelled), DependencyRevision, Payload (opaque to Processing), Diagnostics`.
- **Ownership:** payload owned by the result until published, then by the derived-data slot. **Rules:** payload for authoritative edits is a *proposed* write set returned to Orchestration for commit; Processing never commits geometry.
- **Tests:** stale publication impossible; payload ownership transfer.
- **Open:** payload typing mechanism.

## PR-05 CancellationToken

- **Purpose:** cooperative cancel signal.
- **Model:** `IsCancelled`, `Register(callback)`, linked tokens (parent/child), reason code. One-way latch.
- **Ownership:** created by Processing; copies share state. **Threading:** fully thread-safe. **Errors:** n/a. **Tests:** linked propagation; no callback after dispose.
- **Open:** deadline/timeouts.

## PR-06 JobPriority

- **Model:** class (`Interactive`, `Visible`, `Gameplay`, `Background`, `Maintenance`) + numeric weight (distance, importance) + age for starvation. **Ownership:** Processing defines; submitters fill. **Tests:** no starvation; interactive preempts queueing not running work.
- **Open:** exact weighting; per-platform policy tables.

## PR-07 DependencyGraph registration

- **Purpose:** declare derived-data slots and edges.
- **Model:** `RegisterSlot(SlotId, inputs kinds, provider)`, `Invalidate(DirtyRegion)`, `Query(slot) → {Valid | Stale | Computing | Missing}`.
- **Ownership:** Processing. **Rules:** cycles rejected; registration owned by the module producing the derived data. **Tests:** cycle detection; invalidation matrix by ChangeType.
- **Open:** provider ownership for Surface Mesh, Collision, Hitbox, LOD.

## PR-08 SpatialHierarchy / BvhUpdate / BoundsUpdate

- **Purpose:** derived spatial structures and bounds (Core defines base bounds types).
- **Model:** `SpatialHierarchy` (sparse octree per Layer/Asset, BVH for intersection/collision); `BoundsUpdate` result: World AABB, OBB, Sphere with DependencyRevision.
- **Rules:** Processing never mutates Local AABB. **Revision:** published only if inputs unchanged. **Threading:** queries on published immutable versions; updates swap atomically.
- **Tests:** incremental vs full rebuild equivalence; stale rejection; empty bounds.
- **Open:** Octree vs BVH split of responsibility for broad-phase; query API belongs to a future Physics/Query contract (not in AU-1.0 scope).

## PR-09 Engine ports (defined here, implemented by Engine Adapters)

- `IJobExecutor`, `IAllocator`/memory budget, `IClock`, `IMainThreadBudget`. Conceptual only; no engine API. **Open:** exact set.
