# Cube — Geometry Core Contracts

**Status:** DRAFT 0.1 — specified for approval; none approved.
**Owner of this file:** Integration Owner. **Implementing module:** Geometry Core (Core Agent).
**Basis:** AU-1.0 §4–§10, §23. Common conventions: see `README.md`.
Notation is conceptual. No language-specific signature is final.

Field order per contract: Purpose · Model · Ownership · Lifetime · Access · Revision · Dirty · Threading · Cancel · Errors · Serialization · Compat · Tests · Open.
"Defaults" = the common conventions in `README.md` §2 apply unchanged.

---

## C-01 GeometryAsset

- **Purpose:** primary editable resource; container of Layers, Junctions, CompositeReferences.
- **Model:** `AssetId, AssetToken, Metadata, Bounds(Local AABB union, derived), Layers[], Junctions[], CompositeReferences[], DependencyMetadata`.
- **Ownership:** Core owns the live Asset. Others hold `GeometryAssetHandle` (opaque, non-owning).
- **Lifetime:** Handle valid while the Asset is loaded; stale handle ⇒ `InvalidAsset`.
- **Access:** reads via read views; mutation only through the Editing pipeline commit primitive.
- **Revision:** Asset revision (see semantics §2).
- **Dirty:** Asset-level changes (layer add/remove, junction change) emit DirtyRegion ChangeType Topology/Geometry of the affected layer; layer add/remove emits a structural event.
- **Threading:** read views thread-safe against snapshots; live structure guarded by commit gate.
- **Cancel:** n/a. **Errors:** `InvalidAsset`, `OutOfBounds`.
- **Serialization:** via Asset Manifest (P-01); Core defines no file format.
- **Compat:** unknown optional metadata preserved.
- **Tests:** layer add/remove atomicity; handle invalidation; metadata preservation.
- **Open:** Asset-level metadata schema.

## C-02 GeometryLayer

- **Purpose:** one authoritative geometry representation with own config.
- **Model:** `LayerId, AssetId, LayerType, LayerToken, Transform, ResolutionProfile, MaterialMapping, Channels[], Storage(BrickStorage), Modifiers(ModifierStack ref), CollisionProfile, LayerMetadata`.
- **Ownership:** Core. One authoritative source per Layer; derived mesh/collision are never authoritative.
- **Lifetime:** within Asset. **Access:** as C-01. **Revision:** Layer revision.
- **Dirty:** per ChangeType (Geometry, Material, Attribute, Transform, Topology, Resolution).
- **Threading/Cancel:** as C-01. **Errors:** `InvalidLayer`, `UnsupportedGeometryOperation`.
- **Serialization:** via manifest + brick storage. **Compat:** unknown channels preserved.
- **Tests:** authoritative vs derived separation; transform change does not rebuild bricks.
- **Open:** CollisionProfile detail; owner of collision data schema.

## C-03 GeometryLayerType

- **Purpose:** discriminates representation; supports built-in and registered extension types (AU-1.0 §21).
- **Model:** built-in `CubicVoxel, SDF, PolygonMesh` + `ExtensionTypeId` registered with capability descriptor.
- **Ownership:** Core owns the registry. **Lifetime:** process lifetime. **Access:** read-only after registration.
- **Revision:** registry schema version. **Dirty:** n/a. **Threading:** registry frozen after startup (Open).
- **Errors:** `UnsupportedGeometryOperation`. **Serialization:** stable numeric/string id stored in manifest. Unknown required type ⇒ controlled error; unknown optional ⇒ preserved opaque.
- **Compat:** ids never reused. **Tests:** id stability; unknown type handling.
- **Open:** extension registration rules; binding to backend capability.

## C-04 GeometryTransform

- **Purpose:** placement of a Layer/Part; separate from geometry data (AU-1.0 §5.1).
- **Model:** `Translation(Vector3), Rotation(Quaternion), Scale(Vector3)`; derived `Matrix4x4` cache; optional general affine for special cases with explicit `HasShear` flag; `HandednessConvention = RightHanded, X right, Y up, Z forward`.
- **Ownership:** value type owned by Layer/Part. **Lifetime:** by value.
- **Access:** immutable value; changes via transaction (ChangeType Transform).
- **Revision:** increments Layer revision; bricks untouched.
- **Dirty:** Transform → invalidates World AABB/OBB/Sphere, BVH, collision world data, not Local AABB or bricks.
- **Threading:** value-safe. **Errors:** `UnsupportedGeometryOperation` for singular/unsupported transform without explicit Bake.
- **Rules:** quaternion normalisation policy (tolerance defined in contract); negative scale triggers winding/normal handling flag; shear never dropped by implicit TRS decomposition.
- **Serialization:** TRS stored exactly as authored; matrix not stored. **Compat:** precision type is Open.
- **Tests:** inverse-transform property; negative scale winding; shear detection; singular matrix.
- **Open:** f32 vs f64 for translation; normalisation epsilon.

## C-05 GeometryBounds

**Status (DEC-079):** Owner-Approved as the minimal PS-2 contract, Implemented by CORE-15 (`crates/cube-core/src/bounds.rs`). Decided: integer `i64` half-open boxes at one level, `Empty` as a distinct state, exact checked arithmetic (overflow is an error), conservative growth, no world-size limit; derived OBB and bounding sphere are not part of it (transforms are axis rotations, so the world box is exact). Unknown data is expressed by the cell state (`NotLoaded`, `LoadFailure`) or an error, never by `Empty`.

- **Purpose:** spatial extent types and basic checks.
- **Model:** `LocalAABB` (primary), derived `WorldAABB, OBB, BoundingSphere`, state `Empty`.
- **Ownership:** Core defines types and synchronous ops (validity, union, intersect, transform-to-world basic). Processing computes derived bounds asynchronously and BVH.
- **Lifetime:** value. **Access:** read-only values. **Revision:** Local AABB is tied to Layer revision; derived bounds carry their DependencyRevision.
- **Dirty:** Geometry/Topology/Transform/Resolution changes mark bounds stale.
- **Threading:** value-safe. **Errors:** invalid bounds (min>max) ⇒ validation failure.
- **Serialization:** Local AABB stored; derived not stored (recomputable). **Compat:** integer vs float representation of bounds is Open.
- **Tests:** empty bounds algebra; transform to world; conservative growth under edit.
- **Open:** exactness (tight vs conservative) of Local AABB after removals.

## C-06 GeometryRegion

- **Purpose:** logical region of an operation (AU-1.0 §8).
- **Model:** `Space (Layer|Asset|World), Shape (AABB initial; others Open), Bounds`, four roles: `ReadRegion, WriteRegion, DependencyRegion, ProcessingRegion`. Operations: intersect, union, expand-by-radius, enumerate-intersecting-BrickKeys.
- **Ownership:** value type, Core. **Lifetime:** by value.
- **Revision:** region revision is derived (semantics §2). **Dirty:** DirtyRegion = actual changed data ⊆ WriteRegion. ProcessingRegion expansion never becomes Dirty.
- **Threading:** value-safe. **Errors:** `OutOfBounds`. **Serialization:** used in journal/delta/network payloads; encoding versioned.
- **Tests:** negative coordinates; brick enumeration at boundaries; expansion does not alter dirty.
- **Open:** non-AABB shapes; coordinate space conversion API.

## C-07 BrickKey

- **Purpose:** unique storage address.
- **Model:** `LayerId, BrickCoord(signed integer x,y,z), RefinementLevel`.
- **Ownership:** value. **Rules:** base brick 16³; local index order is **provisional** and **not exposed**; all clients use addressing helpers, never the formula.
- **Revision:** key of Brick revision. **Errors:** `OutOfBounds`.
- **Serialization:** exact integers, stable byte layout (versioned). **Compat:** coordinate integer width Open.
- **Tests:** negative-coordinate floor-division mapping; round-trip world→brick→local→world.
- **Open:** i32/i64; whether RefinementLevel is part of the key or of a sub-address.

## C-08 BrickStorage

- **Purpose:** runtime brick container per Layer (sparse, hash-map direct access; octree for spatial organisation).
- **Model:** operations (conceptual): `GetBrickView(key) → BrickResult`, `AcquirePin(keys) → pins`, `Commit(writeSet)` (only through the commit primitive), `Enumerate(region)`, `ExportBrick(key) → SerializableView`, `ImportBrick(...)`.
- **BrickResult** distinguishes: `Present`, `KnownEmpty`, `NotLoaded`, `OutsideBounds`, `LoadFailure` (never merged).
- **Ownership:** Core owns payload. Consumers hold **read views or pins**, never mutable pointers.
- **Lifetime:** copy-on-write, ref-counted; pin keeps a version alive after replacement.
- **Access:** writes only via commit primitive invoked by Orchestration; backends receive a scoped writable brick *within* a transaction.
- **Revision:** Brick revision per write; commit increments.
- **Dirty:** commit returns the DirtyRegions produced.
- **Threading:** concurrent readers; one commit gate per Asset.
- **Cancel:** n/a. **Errors:** `ResourceUnavailable`, `PersistenceFailure` (load failure state), `InvalidLayer`.
- **Serialization:** Core provides export/import views only; format belongs to Persistence (P-02). History has **no** brick storage of its own; it stores deltas that refer to this contract.
- **Compat:** channel storage modes Dense/Sparse/Constant/Bitset selectable per channel.
- **Tests:** COW isolation; pin lifetime; missing-state distinction; concurrent commit/read; sparse/dense equivalence.
- **Open:** SoA/AoS layout details; storage index order; compression (Open per AU-1.0 §24).

## C-09 GeometryChannel

- **Purpose:** typed attribute channels (Occupancy, MaterialID, Density, SDF, Normal, Color, UV, Emission, Roughness, custom).
- **Model:** `ChannelId (registry), ValueKind (Categorical | Continuous | Bitfield | Opaque), StorageMode, Precision, DefaultValue`.
- **Ownership:** Layer. **Rules:** categorical channels (MaterialID) are **never interpolated**; continuous channels may be.
- **Revision:** part of Brick revision. **Dirty:** ChangeType Attribute/Material/Geometry by channel class.
- **Errors:** `UnsupportedGeometryOperation`, validation errors. **Serialization:** unknown optional channels preserved as opaque; unknown required channels ⇒ controlled error.
- **Tests:** categorical-not-interpolated; opaque round-trip; adaptive width correctness.
- **Open:** typed Attribute Registry schema; adaptive occupancy/material width thresholds; SDF precision.

## C-10 MaterialMapping

- **Purpose:** Global Material Library → Asset Palette → Layer mapping (AU-1.0 §21).
- **Model:** `PaletteEntry(LocalId → GlobalMaterialRef)`, `LayerMapping(LayerLocalId → PaletteId)`.
- **Ownership:** Asset owns palette; Layer holds mapping. Global library is external and read-only.
- **Revision:** palette change increments Layer/Asset revision; Dirty ChangeType Material.
- **Errors:** unresolved global ref ⇒ validation failure with severity; **Serialization:** manifest. **Tests:** remap on boolean/merge; palette boundary widths.
- **Open:** global library interface (not yet specified anywhere); width adaptation policy.

## C-11 ResolutionProfile

- **Purpose:** per-Layer discretisation.
- **Model:** `VoxelSizeMeters` (primary), derived `VoxelsPerUnit`, `BaseBrickSize=16`, `MaxRefinementLevel`, backend-specific sampling params (opaque, backend-owned schema). Resolution is **not** part of Transform.
- **Ownership:** Layer. **Revision:** Layer revision; Dirty ChangeType Resolution. **Errors:** `InvalidResolution`.
- **Serialization:** manifest. **Tests:** conversion exactness; change requires Resampling transaction.
- **Open:** allowed value sets; floating precision of VoxelSize.

## C-12 RefinementRegion

- **Purpose:** localised higher resolution without changing base brick size.
- **Model:** `RegionId, LayerId, Region, Level, TransitionPolicy (Open), RefinementBricks[] via BrickStorage with Level>0`.
- **Ownership:** data model: Core. Creation/removal/transition logic: Resolution Agent via transactions.
- **Revision:** Layer revision + per-brick. **Dirty:** ChangeType Resolution (plus Geometry when data resampled).
- **Rules:** common coordinate system; no cracks at transitions; undo-able.
- **Errors:** `InvalidResolution`, `ConversionRequired`. **Tests:** boundary continuity; repeated refine/unrefine; invalidation.
- **Open:** transition algorithm; max level; hierarchy shape.

## C-13 Junction

- **Purpose:** explicit relation between two Layers (Separate, Blend, Union, Boolean).
- **Model:** `JunctionId, LayerA, LayerB, Mode, Region, Parameters, JunctionToken`.
- **Ownership:** Asset (data), Core. Evaluation of Blend/Union (derived surface) is Processing derived data; Boolean mode defers to Boolean module via explicit operation. **No automatic merge.**
- **Revision:** Asset revision + junction token. **Dirty:** Geometry (derived surface only unless Boolean operation executes).
- **Errors:** `IncompatibleGeometryTypes`, `ConversionRequired`. **Serialization:** manifest. **Tests:** surface-union vs boolean-merge distinct; removal when layer removed.
- **Open:** Blend parameter schema.

## C-14 CompositePart (data model)

- **Purpose:** part record within a composite structure.
- **Model:** `PartId, GeometryAssetRef, LocalTransform, ParentRef?, Durability, Metadata, CollisionProfile, AttachmentPoints[], GameplayState`.
- **Ownership:** **data model: Core (flat layers + composite reference data)**; **operations: Composite Agent** — see ACR-003 (placement pending confirmation).
- **Revision:** Asset revision of the composite asset; parts reference assets by id + expected token.
- **Dirty:** Transform (propagation), Topology (attach/detach).
- **Errors:** `InvalidAsset`; reference to missing asset ⇒ typed error, not crash. **Serialization:** via manifest (references, not copies). **Tests:** reference restoration after reload; reparent preserves world transform when requested.
- **Open:** durability / gameplay state ownership (belongs to gameplay layer or this contract?).

## C-15 AssemblyGraph (data model)

- **Purpose:** non-hierarchical relations between parts.
- **Model:** `Nodes(PartId)`, `Edges(AttachmentPoint↔AttachmentPoint, Constraint)`, dependency edges. Acyclic constraint where required.
- **Ownership/Placement:** as C-14. **Revision/Dirty:** as C-14. **Errors:** cycle detected ⇒ typed validation failure. **Tests:** cycle rejection; transform propagation order; destroy part cascade.
- **Open:** constraint solver scope (belongs to Composite, outside Core).

## C-16 RevisionToken (foundation) — ACR-001

- Specified in `revision-and-snapshot-semantics.md`. **Owner:** Core Agent (primitive type and comparison); semantics owned by Integration Owner.
- Placement note: AU-1.0 §23 lists this under Editing; it is consumed by Core and Processing, which cannot depend on Editing. Moved to Core foundation (ACR-001).

## C-17 OperationId (foundation) — ACR-001

- **Purpose:** identify an editing operation across transaction, history, network, error context.
- **Model:** opaque identifier; format Open (OD-R8). Equality only. Never ordered, never a revision.
- **Ownership:** value. **Serialization:** journal/network/delta/error. **Tests:** uniqueness, idempotent replay.
- **Placement:** Core foundation for the same reason as C-16.

## C-18 DirtyRegion (foundation) — ACR-001

- **Purpose:** record of **actually changed** data.
- **Model:** `AssetId, LayerId, Region(min/max), ChangeType (Geometry|Material|Attribute|Transform|Topology|Resolution), SourceToken, OperationId?`.
- **Rules:** tracked inside Brick (sub-brick granularity); Processing consumes and expands as needed but never feeds expansions back as dirty.
- **Ownership:** value produced by the commit primitive. **Threading:** value-safe. **Serialization:** may appear in journal and network messages.
- **Placement:** Core owns the data type (AU-1.0 §10 requires brick-level tracking by Core); listed under Processing in AU-1.0 §23 (ACR-001).
- **Tests:** no false dirty from no-op writes; correct merge of adjacent dirty regions; classification of ChangeType.
- **Open:** sub-brick granularity (bitmask vs boxes).

## C-19 Result / Error envelope (foundation) — ACR-001

- **Model:** `Result<T> = Ok(T) | Error`; `Error = {StableCode, Module, Context(structured, no memory references), OperationId?, RecoveryPolicy, Severity, Category}`; Category extends the 14 categories of Spec §5.3.
- **Ownership:** Core owns envelope shape and base codes; each module owns its code range; Editing API owns the public category set.
- **Rules:** no exceptions for expected failures; exceptions only for unexpected critical failures. **Tests:** code stability, context safety.
- **Open:** code numbering scheme.
