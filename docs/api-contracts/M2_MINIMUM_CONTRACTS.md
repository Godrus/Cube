# Minimum technical contracts for M2-T

**Status:** **Proposed** / **Pending Owner Approval**. No contract is approved by this document; no content of an absent specification is invented. A "temporary contract" below is a **proposal for a Provisional registration** under plan 3.1/3.2 (basis, assumptions, affected modules, risks, revisiting condition) with an explicit expiry. Nothing becomes effective without a registry entry of the Project Owner (plan 3.3).

## 1. What is known about the two absent specifications

| Specification | State in the repository | What exists instead |
|---|---|---|
| **Geometry Processing Specification** | absent (blocker B-1); AOD-24 states it "may not be replaced by a general assumption before PR-01..PR-08 are approved" | Draft contracts PR-01...PR-08 in [`docs/api-contracts/processing-contracts.md`](processing-contracts.md) (Proposed; text derived from the reconstructed AU-1.0, not from the specification) |
| **Voxel Architecture 1.0** | absent (blocker B-1) | the spike's brick model as candidate baseline BL-1 (AOD-24, not approved); Draft contracts C-07...C-12 (PC-1 is the Provisional minimum slice C-07, C-08, C-09) |

Both statements are checked at HEAD `362a124`. The content of these two documents is **unknown** and is not reconstructed here. The "temporary contracts" therefore speak only about *what M2-T relies on* and *what it must not assume*.

## 2. Verdict per area (A, B, C as in the brief)

A = the full specification must exist before M2-T. B = a limited temporary contract is enough for M2-T. C = not needed for M2-T, may move.

| # | Area | Verdict | Temporary contract | Why |
|---|---|---|---|---|
| 1 | Geometry Processing Specification | **B** (and the full document before M3, see section 4) | TC-01 | M2-T uses five of the eight Draft contracts (snapshot, dependency revision, task, result, cancellation); the missing document is a risk to be registered, not a reason to wait; AOD-24's condition needs the owner's explicit reading (ODM-02) |
| 2 | Voxel Architecture | **B** (full document before the first format approval and before M3 content) | TC-02 (and TC-08 for the stored form) | PC-1 is already the plan's mechanism; storage format stays experimental |
| 3 | Coordinate system | **A for the decision, B for its status** | TC-03 | key and `VoxelPos` widths, level semantics, floor rule must be decided **before** `CORE-02` (GOV-01 item 10); they are then Provisional (PC-1) |
| 4 | World / chunk / brick addressing | **B** | TC-04 | brick = Core `CellKey`; a chunk is a streaming/generation concept (spike: a column of 2 bricks); vertical extent and chunk layout are undecided and must be fixed for M2-T |
| 5 | Rendering interface | **B** | TC-05 | the interface between providers, the client and the engine adapter; engine-neutral; floats allowed (rendering-only data) |
| 6 | Collision interface | **B** | TC-06 | movement and picking need voxel queries; BVH and spatial hierarchy are not needed |
| 7 | World generation interface | **B** | TC-07 | WG-01...WG-10 are Draft; M2-T needs a subset |
| 8 | Persistence interface | **B** | TC-08 | snapshot-only experimental format v0 |
| 9 | Player / world interaction | **B** | TC-09 | new; reuses `IEditingService` |
| 10 | Runtime threading and async | **A** (small) | TC-10 | needed before `PROC-03`; it is a rule set, not a large specification |
| -- | Full persistence format, protocol beyond in-process, SDF/mesh representations, dedicated host, device budgets | **C** | none | not used by M2-T |

## 3. The temporary contracts

Common fields for each: **scope**, **limits**, **forbidden assumptions**, **compatibility** with the future architecture, **migration path**, **expiry / review trigger**, **approver**. "Approver" is always the Project Owner (registry entry); the Integration Owner prepares and validates. Existing source contracts are cited by id; "open" items are those the Draft itself lists as open.

### TC-01 Processing subset (PR-01...PR-05)
* **Scope:** immutable snapshot of a brick neighbourhood with an apron (PR-01); derived-slot dependency revision by input stamps (PR-02); executor task and priority classes (PR-03/PR-06 minimal); result with stale rejection (PR-04); cancellation token (PR-05).
* **Limits:** only what meshing, collision and generation streaming need; caps are configuration constants marked tunable.
* **Forbidden assumptions:** any content of the absent specification; granularity of PR-02 beyond per-brick; memory-cap policy (OD-R4); BVH/spatial hierarchy (PR-08); that PR-01...PR-08 are approved.
* **Compatibility:** the Draft text is unchanged; the implementation sits behind the Processing crate's public API; no other crate imports internals.
* **Migration:** at the arrival of the specification, diff PR-01...PR-08 against it (gate RG-2); a budgeted rework task exists (not estimated).
* **Expiry:** the earlier of (a) approval task `GOV-07` accepted, (b) the start of M3 content work.
* **Approver:** Project Owner (ODM-02). **Open question for the owner:** does a Provisional registration satisfy AOD-24's condition? If not, only the supplied specification unblocks the path ([`M2_TASK_GRAPH.md`](../archive/m2-delivery-review/M2_TASK_GRAPH.md) section 6).

### TC-02 Voxel representation baseline (PC-1 = C-07, C-08, C-09)
* **Scope:** brick-of-cells storage with constant/dense encodings, `u16` material channel, optional quantised density; Euclidean division; named-constant brick edge 16 (BL-1).
* **Limits:** the Voxel layer representation only; SDF and mesh representations are not implemented in M2.
* **Forbidden assumptions:** that the edge, key widths or the density quantisation are final; that any persisted or wire format depends on them beyond TC-08's experimental label.
* **Compatibility:** Core's closed representation enum and the layer/transform model are kept, so adding SDF/mesh later is additive.
* **Migration:** revalidation gate RG-1 (diff against the Voxel Architecture); a data migration is expected if widths change.
* **Expiry:** RG-1 passed, or the first format approval (`GOV-10`), whichever is first; PC-1 never becomes Frozen without it (plan 3.1).
* **Approver:** Project Owner (GOV-02; ODM-03).

### TC-03 Coordinate system v0
* **Scope:** `CellKey` (= brick key), `VoxelPos`, `LocalPos`, level semantics, floor-division rule.
* **Limits / decisions needed before CORE-02:** widths of key and voxel position (they must be decided together; the spike's `i32` brick coordinate times 16 overflows `i32`), direction of `level`, the stated world bound (GOV-01 item 10, decisions D-K, ODM-11).
* **Forbidden assumptions:** an unbounded world with a fixed-width key; that level mapping is known.
* **Compatibility:** widths are named constants; the persisted key encoding is fixed-width and versioned (decided with TC-08).
* **Migration:** change of widths = data migration; therefore decided once, early.
* **Expiry:** RG-1.
* **Approver:** Project Owner (GOV-01 item 10).

### TC-04 World extent, chunk and brick addressing v0
* **Scope:** a chunk is a column of N bricks in the streaming/generation layer (not in Core); the horizontal chunk coordinate is the floor of the voxel coordinate by the chunk edge; N (vertical extent) is a content parameter.
* **Limits:** M2-T fixes one vertical extent for the generator; no vertical streaming.
* **Forbidden assumptions:** that N=2 (the spike's 32 voxels) is enough for the M2 terrain; that gameplay code knows chunk size.
* **Compatibility:** Core sees bricks only; changing N changes the generator version, not Core.
* **Migration:** generator version bump rehearsal (WGEN version policy).
* **Expiry:** M3 content scope decided.
* **Approver:** Project Owner (ODM-11), Integration Owner validates.

### TC-05 Rendering interface v0
* **Scope:** a provider produces, per mesh section, vertex and index data with material ids and section-local integer positions; the client schedules builds closest-first and drops stale results; the adapter uploads buffers.
* **Limits:** `faces` provider only; no LOD, no AO, no atlas.
* **Forbidden assumptions:** that the buffer layout is a persisted or network format; that the engine's types appear in Core or Processing.
* **Compatibility:** providers register through the Processing provider registry (PR-02/PR-07 subset), so greedy and LOD providers can be added.
* **Migration:** none expected; layout changes are local to provider and adapter.
* **Expiry:** first additional provider or the device checkpoint MAC-1, whichever is first.
* **Approver:** Project Owner for the contract text; Integration Owner validates conformance.

### TC-06 Collision and query interface v0
* **Scope:** solid-test of a point or box against a snapshot; ray walk over bricks (`TGAP-01`).
* **Limits:** voxel representation only; no BVH; queries run on the server side view.
* **Forbidden assumptions:** that gameplay uses a physics engine; that query results are stable across revisions (a query names the snapshot revision it used).
* **Compatibility:** the home of the query API (open in PR-08) is decided in `TGAP-01`; either Processing or the voxel backend.
* **Migration:** local.
* **Expiry:** when the owner of PR-08 is decided and PR-08 approved.
* **Approver:** Project Owner (home decision).

### TC-07 World generation interface v0
* **Scope:** pure `generate(seed, generator version, chunk coordinate)`; version registry; materialiser through `SystemCommit`; deterministic stable hash per chunk.
* **Limits:** from WG-01...WG-10 only the parts used by `WGEN-01..04`; one generator version at a time.
* **Forbidden assumptions:** any floating point in world-defining math (the spike generator is integer-only, verified); that terrain depends on entity randomness; that generated data is a user operation (AOD-16).
* **Compatibility:** the pure generator is read-only against Core; the materialiser is the only writer for generated data.
* **Migration:** generator version bump with pinned hashes.
* **Expiry:** `GOV-21` accepted.
* **Approver:** Project Owner (GOV-21 or a Provisional entry; ACR-004 B/C/E via GOV-11).

### TC-08 Persistence v0: snapshot-only, experimental format
* **Scope:** world record (`WorldSeed`, `GeneratorVersion`, config), snapshot of modified bricks, atomic manifest publish with checksums; load order base, snapshot (journal tail empty). In M2: journal, recovery and compaction are added (`PERS-04..06`).
* **Limits:** **deviates from AOD-21's rule that the journal is the primary record**; allowed only as a registered temporary contract; the format carries a version marker and the label "experimental, no compatibility promise".
* **Forbidden assumptions:** that saves from M2-T are readable by later versions; that this format is the approved format (`GOV-10`); that other modules read the files.
* **Compatibility:** the load order is a prefix of AOD-21's order; the store sits behind `ISnapshotStore`/`IBaseProvider`.
* **Migration:** none promised; M2-T saves are disposable; from M2 on, versioned records with a migration hook (`PERS-07` Later).
* **Expiry:** acceptance of M2, or the first format approval.
* **Approver:** Project Owner (ODM-03).

### TC-09 Player / world interaction v0 (with gameplay record store and messages)
* **Scope:** a player request to destroy or place a voxel is an operation with origin Player through `IEditingService`, validated for reach on the server; game state is stored through an opaque, versioned record store (`TGAP-02`); the protocol carries intents upward and player, chunk, and (M2) entity, inventory and event messages downward (`TGAP-03a/b`).
* **Limits:** single-player, one player, in-process transport.
* **Forbidden assumptions:** gameplay types inside Core, Processing, Persistence or the Editing API; persistence interpreting game records; client-side authority.
* **Compatibility:** sessions, authentication and multiple players are additive (`SRV-02`, M4).
* **Migration:** record schemas are versioned from the start.
* **Expiry:** M3 gameplay scope or M4 multiplayer design, whichever is first.
* **Approver:** Project Owner (module and contract: ODM-05, ODM-07).

### TC-10 Runtime threading and async contract
* **Scope:** no async runtime in Core, Processing or backends (std-thread executor with named classes); the engine main thread never blocks: generation, meshing, save and load run on executor threads and deliver results by polling; blocking points are listed in the contract; the server tick is a single writer.
* **Limits:** the server/network layer's runtime is not decided here (GOV-12/22).
* **Forbidden assumptions:** that a blocking call is acceptable "briefly" on the main thread; thread-affine engine objects used off the main thread.
* **Compatibility:** consistent with D-E parameter h (GOV-01 item 14) and the Draft PR-03/PR-05.
* **Migration:** none.
* **Expiry:** reviewed when the first device checkpoint (MAC-1) measures thread behaviour.
* **Approver:** Project Owner (ODM-07); needed before `PROC-03`.

## 4. When the full specifications are needed

| Specification | Needed at the latest | Reason |
|---|---|---|
| Geometry Processing Specification | before PR-01...PR-08 are Owner-Approved (`GOV-07`) and before M3 content work | production merges that rely on Processing must eventually rest on an approved contract; TC-01 expires |
| Voxel Architecture | before the first persistence format approval (`GOV-10`) and before M3 content that must survive versions | formats and PC-1 cannot be frozen without it |
| Full persistence format | before any save is promised to be loadable by a later build (not M2) | |
| Numeric policies for Mesh/SDF/gameplay (D-T classes 3-5) | before those representations or gameplay state are approved | [`GOV-01-DECISION-REGISTER.md`](../archive/gov01/GOV-01-DECISION-REGISTER.md) item 11 |

## 5. Guard against a temporary contract becoming permanent

1. Every TC has an entry in the Provisional register with its expiry; the register is part of [`PROJECT_STATUS.md`](../archive/PROJECT_STATUS.md).
2. `archcheck` rule (proposal): production crates may reference only Owner-Approved or Provisional contracts, and a Provisional contract carries its expiry in a machine-readable list; a CI check fails when an expiry condition is met and the entry is still open.
3. Provisional contracts never become Frozen without revalidation and the owner (plan 3.1).
4. Tests that depend on a Provisional contract are tagged `provisional`.
