# Cube Production Architecture Plan v2 (proposal)

**Status:** **Proposed**. Revision v2 of `production-architecture-plan.md`. It is not a normative document and approves nothing: no contract, ACR, AOD, blocker closure, merge or start of implementation. All owner decisions it needs are listed in `architecture-blocker-decisions.md`; nothing in this plan becomes a decision without a decision-registry entry made by the project owner (section 3.3).
**Location:** prepared in a scratchpad workspace outside the repository; proposed repository path `docs/production/`.
**Document set (all v2):** `production-architecture-plan.md` (this), `module-boundaries.md` (contains the **Authoritative Dependency Graph, ADG**), `implementation-roadmap.md`, `agent-task-breakdown.md`, `architecture-blocker-decisions.md`, `integration-and-qa-plan.md`, `production-plan-revision-v2-report.md`.
**Single-source rule:** stage numbers, task ids, task dependencies and gate conditions are defined **only** in the ADG (`module-boundaries.md`, part C). Every other document refers to the ADG and never restates a dependency. The ADG is generated from one data file and machine-checked (acyclic, no unknown nodes, no stage inversion, one owner per crate); the check result is in the report.
**Evidence labels:** *Fact* (checked in code or documents), *INFERENCE*, *Proposal*.

## 1. Conventions and canonical identifiers

* **Canonical stage names are `PS-0 … PS-14` (Production Stage).** A bare "Stage 11" or "Gate 7" is not used anywhere in v2; every reference names its system: `PS-n` (this plan), `QA-Pack stage n`, `Roadmap-v1 gate n`.
* **Gate nodes** `G0 … G14` close the stages `PS-0 … PS-14` (membership and conditions: ADG parts C and D). **Device gates** `DG-1…DG-3` are separate tasks for physical-device validation (section 2.3 of the roadmap). **Revalidation gates** `RG-1` (Voxel Architecture) and `RG-2` (Geometry Processing Specification) are external-input gates (section 4.4).
* **Edge kinds in the ADG:** `S` = *start-after* (the dependency's public interface is frozen, work may begin in a branch); `I` = *merge-after* (the dependency is merged and accepted). A task merges only after the union of its `S` and `I` dependencies.
* **Contract ids:** C-xx (Core), PR-xx (Processing), E-xx (Editing), P-xx (Persistence), WG-xx, SC-xx, WP-xx, SR-xx (Draft additions). Decision ids: `D-…` in `architecture-blocker-decisions.md`.

## 2. Canonical stage system and mapping from the old numberings

### 2.1 Canonical stages (architectural order unchanged from v1; only ambiguity removed)

| Stage | Name |
|---|---|
| PS-0 | Governance and Architecture Freeze |
| PS-1 | Geometry Core Foundation (closes P0-alpha only) |
| PS-2 | Processing Foundation |
| PS-3 | Editing API and Contracts |
| PS-4 | Editing Orchestration |
| PS-5 | Geometry Backends and Derived Providers |
| PS-6 | Tools: Sculpting and Boolean |
| PS-7 | History |
| PS-8 | Composite Geometry |
| PS-9 | Resampling and Resolution Refinement |
| PS-10 | Persistence |
| PS-11 | World Generation (11A pure generator; 11B integration) |
| PS-12 | Client Runtime and Engine Adapter |
| PS-13 | Server Runtime and Networking |
| PS-14 | Full QA and Release Validation (with device gates DG-1…DG-3) |

### 2.2 Mapping table

| Old reference | Meaning | Canonical |
|---|---|---|
| v1 plan "Stage 0-14" | same names | identical numbers `PS-0 … PS-14`; v1 text "Stage 12 before 13" kept |
| QA-Pack stage 0 | Decisions and tier approval | PS-0 |
| QA-Pack stage 1 | Geometry Core | PS-1 |
| QA-Pack stage 2 | Processing foundations | PS-2 |
| QA-Pack stage 3 | Editing API and Contracts | PS-3 |
| QA-Pack stage 4 | Editing Orchestration | PS-4 |
| QA-Pack stage 5 | Backends | PS-5 |
| QA-Pack stage 6 | Tools | PS-6 |
| QA-Pack stage 7 | History | PS-7 |
| QA-Pack stage 8 | Composite | PS-8 |
| QA-Pack stage 9 | Resampling and Refinement | PS-9 |
| QA-Pack stage 10 | Persistence Integration | PS-10 |
| QA-Pack stage 11 | Networking Adapter | PS-13 (with server runtime) |
| QA-Pack stage 12 | Engine Integration Adapters | PS-12 (with client runtime) |
| QA-Pack stage 13 | Full Integration and QA | PS-14 |
| (none) | World generation integration | PS-11 (new, outside the QA Pack) |
| Roadmap-v1 gate 0 (Governance and Sources) | | PS-0 |
| Roadmap-v1 Owner Decision Gate | | PS-0 (tasks GOV-xx) |
| Roadmap-v1 gate 1 (Technology Foundation) | toolchain, CI, devices | PS-0 (GOV-05, GOV-06, TEST-01, XT-01, DEV-01) |
| Roadmap-v1 gate 2 (Geometry Core) | | PS-1 |
| Roadmap-v1 gate 3 (World Generation) | | PS-11 |
| Roadmap-v1 gate 4 (Meshing and Client Adapter) | | PS-5 (providers), PS-12 (client, adapter), DG-1 (device benchmark) |
| Roadmap-v1 gate 5 (Editing and Persistence) | | PS-3, PS-4, PS-7, PS-10 |
| Roadmap-v1 gate 6 (Player and Local Game) | gameplay | **outside this plan** (gameplay layer; open scope item, section 12) |
| Roadmap-v1 gate 7 (Server and Multiplayer) | | PS-13 |
| Roadmap-v1 gate 8 (World Content) | content | **outside this plan** |
| Roadmap-v1 gate 9 (MMO Systems) | | **outside this plan** |
| Roadmap-v1 gate 10 (Mobile Production) | | PS-14 and DG-2, DG-3 |

`docs/architecture/roadmap-v1.md` and the QA Pack are existing normative-candidate documents and are not edited by this revision; if the owner adopts the canonical names, those two documents receive a one-line pointer to this mapping.

## 3. Contract Status Model and Decision Registry

### 3.1 Statuses

Main line: **Proposed → Owner-Approved → Implemented → Frozen**. Side states: **Provisional**, **Rejected**, **Superseded**. The legacy words in `docs/api-contracts/README.md` map as: `Draft` = `Proposed`; `Draft+OD` = `Proposed` with open decisions; `Approved` = `Owner-Approved`.

| Status | Assigned by | Evidence required | Allowed | Restrictions |
|---|---|---|---|---|
| **Proposed** | any author (agent or owner) | the contract text in the repository | design, review, mock implementations, spikes | no production module may claim it; no persisted/wire format may depend on it |
| **Provisional** | project owner (record), set by the Integration Owner | registry entry with the five mandatory fields (3.2) | implementation behind explicit extension points; merge to `master` marked provisional | **cannot become Frozen**; must pass its revalidation gate; downstream modules may use it only if their own tasks list the dependency and accept rework risk |
| **Owner-Approved** | **project owner only** | registry entry with the confirmation source (3.3) | production implementation against the contract | text changes need a change request; an agent never sets this status |
| **Implemented** | Integration Owner | module merged; contract tests listed in the task are green in CI; acceptance criteria evidenced | consumers integrate | public API changes need a change request |
| **Frozen** | project owner on the Integration Owner's recommendation | Implemented + stable for the freeze criteria in the stage gate + for provisional contracts: revalidation gate passed | consumers rely on it; breaking change only via ACR | any change is breaking and needs an ACR and owner approval |
| **Rejected** | project owner | registry entry | none | text kept for history, never implemented |
| **Superseded** | project owner | registry entry naming the replacing contract | none for new code | existing code migrates per the replacing contract's plan |

**Agents never move a contract from Proposed to Owner-Approved.** A document that says "Approved" without a registry entry is wrong and is corrected by the Integration Owner.

### 3.2 Provisional Contracts: mandatory fields

Every Provisional contract needs: (1) **basis** of the temporary decision; (2) **linked assumptions**; (3) **affected modules**; (4) **risks**; (5) **revisiting condition** and the **revalidation gate** (RG-n) that triggers a mandatory review. Registered provisional contracts are listed in section 4.3.

### 3.3 Decision registry

A single file `docs/decisions/decision-registry.md` (proposed; created by the owner's decision GOV-01/GOV-02) records every owner decision: id, date, decision, status, scope, **source of confirmation**, superseded ids. Accepted confirmation sources: (a) an owner-signed commit or pull-request approval in the repository; (b) a decision text written by the owner in the repository. **A chat instruction alone is not a confirmation source**; an agent may transcribe it as `recorded, awaiting confirmation`, and it becomes effective only when the owner countersigns by (a) or (b). *Fact:* AOD-01…24 (`owner-architecture-decisions-v1.md`) were transcribed by an agent from chat; they are therefore `recorded, awaiting owner confirmation` until countersigned (decision GOV-01 covers this).

## 4. P0, P0-alpha and Provisional Contracts

### 4.1 Definitions

* **Full P0** = all 19 Core contracts C-01…C-19 Owner-Approved (and the Provisional ones revalidated).
* **P0-alpha** = the subset the owner may approve **now**, independent of the missing Voxel Architecture: **C-01, C-02, C-03, C-04, C-06, C-16, C-17, C-18, C-19** (core slices only; the open fields named in the contracts, e.g. asset metadata schema and collision profile, are excluded from the approval and stay Proposed).
* **Provisional Contracts PC-1** = **C-07 BrickKey, C-08 BrickStorage, C-09 GeometryChannel**, restricted to the minimum slice needed to store cells (section 5.3).
* **Deferred from PS-1** (separate Core Agent tasks in the stage that consumes them, each with its own approval): C-05 (CORE-15, PS-2), C-10 (CORE-12, PS-5), C-11 and C-12 (CORE-13, PS-9), C-13, C-14, C-15 (CORE-14, PS-8).

### 4.2 Rules

1. **PS-1 may close only on an Owner-Approved P0-alpha**; Provisional contracts are implemented and tested in PS-1 but do not close it by themselves.
2. A Provisional contract **never becomes Frozen automatically**. Freezing requires its revalidation gate (4.4).
3. **P0-alpha is not Full P0.** No document may call P0-alpha "P0". Full P0 completes only after the deferred contracts are Owner-Approved and RG-1 passed.
4. Resolution of the earlier contradiction (v1 D-0 versus PS-1 exit): v1 let Stage 1 "implement C-07…C-12 provisional" while its exit required P0 "Implemented". v2: PS-1 exit requires **P0-alpha Owner-Approved and Implemented**, PC-1 **Provisional and Implemented-Provisional**; C-10…C-12 are not part of PS-1 at all.

### 4.3 Provisional register (proposed entries; effective only after owner approval, GOV-02)

| Id | Contracts | Basis | Linked assumptions | Affected modules | Risks | Revisiting condition and gate |
|---|---|---|---|---|---|---|
| PC-1 | C-07, C-08, C-09 | needed to store any cell; the Voxel Architecture is absent (B-1) | BL-1 (AOD-24 candidate): brick edge 16, i32 coordinates, `level` in the key, `u16` material channel, quantised density channel, Constant/Dense encodings; spike brick model as working model | Core, Processing, all backends, Persistence, WorldGen | rework of key width, channel set, brick edge; persisted-format impact | RG-1: arrival of Voxel Architecture v1.0; mandatory review of PC-1; Persistence format approval (GOV-10) waits for RG-1 |

### 4.4 Revalidation gates

* **RG-1:** Voxel Architecture supplied (external node `EXT-VA-RG1`); the Integration Owner diffs PC-1 and C-10…C-12 against it; the owner records the result. Blocks: Frozen status of PC-1; CORE-12, CORE-13 approval; GOV-10 (format approval).
* **RG-2:** Geometry Processing Specification supplied (`EXT-GPS`); blocks task GOV-07 (PR-01…PR-08 approval) and therefore **G2, the Processing production gate**. AOD-24 forbids replacing the specification with a general assumption.

## 5. Geometry Core: minimal universal model (Proposed)

The model must be small yet not voxel-only. Everything below is a **proposal** for owner approval (decisions D-K, D-T, D-C, D-R1, D-R2, D-R6, D-R8). Spike facts: `GeometryAsset` holds `Vec<VoxelLayer>`; bricks are `Constant|Dense` `u16`; `BrickLookup` already has five states; commit takes a write set for one layer; `voxel_size_m: f32` is stored.

### 5.1 Layer, Representation
* **Purpose:** a named, transformable container of one representation of geometry.
* **Model:** `Layer { id, representation, transform, resolution, data }`; `Representation` ∈ {`Voxel`, `Sdf`, `Mesh`}; `LayerData` is a **closed enum** with one variant per representation: `Voxel(CellStore)`, `Sdf(CellStore)`, `Mesh(PageStore)`. `CellStore` and `PageStore` share the same sparse map shape (`CellKey → Arc<Cell>`). Micro voxels are a `Voxel` layer with a finer `resolution` level, not a separate representation. A new representation requires an ACR (closed enum is deliberate: no plugin registry until C-03's open question is decided).
* **Ownership:** the asset owns layers; layers own their stores; payload cells are shared via `Arc` and immutable once committed.
* **Lifecycle:** structural changes (add/remove layer) are commits that increment the asset revision and are recorded in the commit log.
* **Revision:** asset +1 per commit touching it; layer +1 if its content or properties changed.
* **Extensibility:** new channel kinds and encodings without API change; new representation by ACR.
* **Limits:** no behaviour; no conversion logic.
* **Interaction:** Processing reads via snapshots; Editing never touches layers except through commits.

### 5.2 Channel
* **Purpose:** a typed per-cell value family.
* **Model:** `ChannelId` from a closed built-in set first: `Material(u16)`, `Density` (quantised signed integer with a per-layer scale; no float stored), `Attribute(id, width)`; per channel an encoding `Constant | Dense` (compressed encodings are added without API change).
* **Ownership:** the cell owns its channel arrays; **Lifecycle:** immutable after commit (copy-on-write on edit). **Revision:** cell revision +1 when any channel changes.
* **Limits:** mesh pages use their own payload, not channels. Quantisation parameters are persisted data (D-C).

### 5.3 CellKey (BrickKey)
* **Model:** `CellKey { layer, level: i8, coord: [i32; 3] }`; level 0 is the layer's base resolution; the brick edge is the named constant 16. Euclidean division for negative coordinates (spike-proven).
* **Why `level` now:** refinement (micro voxels) and LOD cannot be added to a frozen key later; the cost is one field that is `0` in PS-1.
* **Validity and missing data:** `CellState` = `Present(payload) | KnownEmpty | NotLoaded | OutsideBounds | LoadFailure` (as in the spike). Only `Present` and `KnownEmpty` are written by commits; the other three are read-side states and are never conflated. A tombstone keeps its revision, which is never reused.
* **Status:** C-07/C-08/C-09 are **Provisional (PC-1)** until RG-1.

### 5.4 Transform
* **Model (D-T proposal):** integer translation, one of 24 axis-aligned rotations and a power-of-two scale for `Voxel`/`Sdf` layers; mesh layers may carry an exact fixed-point affine transform; free rotation is allowed only in derived/render data. **Authoritative and persisted state contains no floating point unless D-T says otherwise** (the spike's `voxel_size_m: f32` is replaced by an integer resolution level).
* **Revision:** a transform change is a layer-level change (layer +1, asset +1). **Interaction:** Composite propagates transforms through operations; Processing treats the transform as a dependency input.

### 5.5 Region
* **Model:** axis-aligned half-open box `[min, max)` of `i32` cell coordinates **in layer space at a stated level**; world-space conversion goes through the layer transform. Non-box shapes are tool-level and do not enter Core.
* **Use:** write-set extents, dirty regions, snapshot regions, region stamps.

### 5.6 Commit
* **Unit:** a **`CommitBatch`** = per-layer write sets for **one asset**, applied atomically (the spike has one layer per write set; a transaction that touches two layers, for example a junction, needs the batch). A write names for each cell `expected` (`Absent` or a revision), the new payload or removal, and the changed region.
* **Algorithm:** validate every expectation (nothing mutated), then apply all writes; +1 per cell, +1 per touched layer, +1 for the asset; all-or-nothing; no-op batches change nothing. Conflicts return the conflicting cells.
* **Concurrency (D-R2 proposal):** immutable `AssetState` behind `Arc`; the commit gate serialises writers; readers take lock-free snapshots.
* **Actor and origin:** every commit carries `ActorKind` ∈ {`User`, `System`, `Recovery`} and an `OperationId`; the handle used decides which kinds are possible (section 6).
* **Outputs:** new tokens, dirty regions with `ActorKind`, post-state of written cells (for History).
* **Limits:** no history, no persistence, no scheduling.

### 5.7 Conversion between representations
Conversion (for example voxelising a mesh) is **never implicit** and never in Core. It is an explicit operation by a tool or backend that reads a snapshot and submits a commit that replaces or creates a layer. Derived views (a render mesh from a voxel layer) are Processing derived data, not conversions.

### 5.8 What the model deliberately omits
No plugin registry for representations, no multi-asset commits, no non-box regions, no compressed encodings, no live-service notions. Each can be added later without breaking the shapes above.

## 6. The mutation boundary (single mutation path)

**Invariant MB-1:** *every change of authoritative geometry state passes through the Mutation Boundary, which is the Core commit gate, reached only through one of four typed entry classes.*

| Entry class | Capability handle | Allowed caller (build-time) | ActorKind | Notes |
|---|---|---|---|---|
| User operation | `UserCommit` | `cube-orchestration` only | `User` | after authorisation (E-02) |
| System operation | `SystemCommit` | `cube-worldgen` Materialiser only | `System` | `Materialise`, `Evict` |
| Persistence recovery | `RecoveryCommit` | `cube-persistence` loader only | `Recovery` | replay under original OperationIds |
| Core-internal | none (private) | `cube-core` only | `System` | structural primitives inside the gate |

**Forbidden:** any mutation of authoritative state outside these classes; a history of changes inside an adapter; independent write paths from `WorldStreamer` or `WorldBridge` (the spike has both: `streamer.rs` direct commits and `set_voxel`; `world_bridge.rs` `edit_voxel`, `fill_box`, `undo`, `redo`; `Editor::replace_asset` is also reviewed as a recovery-class path).
**Enforcement (more than a Cargo check):**
1. **Type-level capability handles:** `SharedAsset` has no public mutating method except through the three handles; handles are created only by the composition root, so an unauthorised crate cannot construct them (compile-fail tests with `trybuild`).
2. **Commit log:** a test-time log records every revision increment with `ActorKind`; a property test asserts there is no increment without a log entry.
3. **archcheck rules (XT-02):** only the allowed crates mention the handle types; adapters and clients contain no history patterns (`undo`, `redo`, `EditBatch`) and no `commit`.
4. **Negative tests (TEST-06):** a user-path request cannot produce a `System` commit; a client crate cannot name a handle; an adapter cannot change revisions; recovery cannot run on a live asset without the recovery handle.
5. **Public-API snapshot** of `cube-core` fails CI if a mutating method appears outside the handles.

## 7. ACR-004: split into five independent parts

Detailed fields (owner, contracts, dependencies, stage, acceptance) are in `architecture-blocker-decisions.md`, section 2. Summary:

| Part | Content | Stage | Closes in |
|---|---|---|---|
| A. Core prerequisites | `ActorKind`, `OperationId`, mutation boundary, system-commit and recovery port types, minimal Core primitives, authoritative-path rules | PS-1 | CORE-07, CORE-08, CORE-09, XT-02, TEST-06 |
| B. WorldGen integration | materialisation, eviction, streaming, interaction with Core | PS-11 (11B) | WGEN-02…04 |
| C. Persistence recovery | recovery, replay, revision interplay, operation journal | PS-10 | PERS-04, PERS-06 |
| D. Revision semantics | creation and propagation of system changes, consistency with commit and recovery | decision in PS-0, implemented across PS-1 / PS-10 / PS-11 | GOV-01 (decision), CORE-06 |
| E. Removal of legacy direct writes | `WorldStreamer`, `WorldBridge`, others | PS-11 (streamer), PS-12 (bridge) | LEG-01, LEG-02 |

**PS-1 closes only part A.** Parts B, C and E are not Core work; part D is a decision recorded in PS-0 whose consequence in PS-1 is only that the commit accepts `ActorKind` with unchanged revision scopes (recommended R-A).

## 8. Persistence production requirements (separate from the spike)

*Fact:* `cube-persistence` is a spike (`CUBEAST1` v1, per-brick CRC, whole-file CRC, tmp → `sync_all` → rename, no directory fsync, no deltas/snapshots/manifest/journal/migration). Its results are **not** evidence of production readiness; only its ideas (CRC, atomic publish) are reused.
Production requirements (AOD-21 architecture is Approved-recorded; formats are not approved): **manifest** (atomic pointer to the recovery point); **snapshot** (records base generator version and revision boundary; invalid until verified); **journal** (append-only, gapless sequence, checksums, tolerates a truncated tail); **replay** (idempotent, original OperationIds, `Recovery` actor); **compaction** (write, fsync, verify, atomic manifest, fsync, then truncate; idempotent); **migration** (versioned, never silent); **atomic write and crash consistency** (every state between steps recoverable); **directory fsync semantics** (publish is not durable until the containing directory entry is made durable; mechanism per platform is the subject of SPK-PERS and DG-2); **versioned OperationId and revision format** (schema version on every record; widths per D-R1, D-R8); **fault injection** (FaultFs, TEST-04) at every numbered I/O step. Persistence depends on the L2 ports and Core, never on History; recovery obtains the base through the `IBaseProvider` port, so it never depends on WorldGen.

## 9. World generation: separated concerns

| Concern | Crate / task | Needs | Parallel with |
|---|---|---|---|
| Pure Generator | `cube-worldgen::generator`, WGEN-01 | read-only Core API frozen (CORE-10), ids | Processing, Editing API |
| Streaming (queue, cancellation, cache, states) | WGEN-02 | WGEN-01, executor and cancellation (PROC-03/04) | Persistence |
| Materialisation | WGEN-03 | system-commit port (CORE-09), ACR-004 B decision (GOV-11) | none before CORE-09 |
| Eviction | WGEN-04 | WGEN-03, storage ports (API-04), journal durability contract | |
| Integration with Core | WGEN-03/04 + ITG-02 | as above | |
| Persistence interplay | ITG-02, PERS-06 | recovery wiring in SRV-01 | |

Streaming, materialisation and any authoritative write depend on the system-commit contracts; only the pure generator may proceed on the read-only API.

## 10. Networking, client runtime, engine adapter: separated concerns

Order: **Transport contract (NET-01) → protocol and session types incl. handshake message types (NET-02) → in-process transport (NET-03) → external transport (NET-04, after SPK-NET and owner decision GOV-12)**; **server authority and runtime (SRV-01, SRV-03)** and **server authentication integration (SRV-02)** are separate from the protocol types; **client runtime** (CLI-00 skeleton in parallel from the Core freeze, production integration CLI-05 only after stable NET and SRV-04); **engine adapter** (ADP-01…03) after the client runtime APIs. Embedded and dedicated server share one runtime (SRV-01 with two hosts) and one conformance suite (TEST-03b).

## 11. Capability coverage

| Capability | Home | Stage | Note |
|---|---|---|---|
| Cubic / Micro Voxel | Core `Voxel` layer + level; `cube-backend-voxel` | PS-1, PS-5, PS-9 | micro = finer resolution level |
| SDF / Density | Core `Sdf` layer + `Density` channel; `cube-backend-sdf` | PS-1, PS-5 | |
| Polygon Mesh | Core `Mesh` layer pages; `cube-backend-mesh` | PS-1, PS-5 | |
| Sparse Brick Storage, adaptive channels, material palette | Core (CORE-03, CORE-12) | PS-1, PS-5 | palette deferred to PS-5 |
| Geometry layers, transforms | Core | PS-1 | |
| Resolution Refinement | Core data (CORE-13), `cube-resolution` | PS-9 | |
| Surface extraction, Greedy, Surface Nets | derived providers in backends (ownership pending O-1) | PS-5 | |
| Boolean, Sculpting | tools | PS-6 | |
| Composite, Assembly Graph | Core data (CORE-14), `cube-composite` | PS-8 | ACR-003 |
| Collision, LOD | providers per backend (pending O-1) | PS-5 | |
| Immutable snapshots, dirty tracking, dependency graph, async | Core + Processing | PS-1, PS-2 | |
| History | `cube-history` | PS-7 | |
| Persistence | `cube-persistence` | PS-10 | |
| World generation | `cube-worldgen` | PS-11 | |
| Multiplayer, server authority | `cube-net`, `cube-server`, `cube-client` | PS-12, PS-13 | |
| Mobile | all; device gates | PS-14, DG-1…DG-3 | |

## 12. Critical path

Two critical paths are reported because one is computed and one is owner-blocked.
1. **Longest engineering chain (computed from the ADG):** `GOV-04 -> GOV-06 -> TEST-01 -> G0 -> CORE-01 -> CORE-03 -> CORE-06 -> CORE-10 -> API-01 -> API-04 -> PERS-01 -> PERS-02 -> PERS-03 -> PERS-05 -> PERS-06 -> QA-P2 -> G10 -> SRV-01 -> SRV-03 -> SRV-04 -> SRV-05 -> G13 -> QA-04 -> G14` (20 tasks; computed by the validator)
2. **Externally blocked chain:** `EXT-GPS → GOV-07 → G2 (Processing production gate) → ORCH-01…05 → G4 → SRV-01 …` and `EXT-VA-RG1 → CORE-12 / CORE-13 / GOV-10 (format approval)`. The Processing gate is on the critical path of every downstream production integration (checked in the validation: G2 is an ancestor of all ORCH tasks, all backend operation tasks and SRV-01).
3. **Parallel from the Core API freeze (CORE-10):** PROC-01…05 start; API-01 starts; WGEN-01 (pure generator); CLI-00; tool contract drafting. **Parallel from PS-0:** SPK-PERS, SPK-NET, TEST-01, XT-01, CI.
4. **Out-of-scope items that still bound the project:** roadmap-v1 gates 6, 8, 9 (gameplay, content, MMO) are not covered by this geometry-focused production plan; the owner decides whether further plans follow.

## 13. Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Geometry Processing Specification absent | G2 cannot close; every production Editing merge waits | owner supplies it (RG-2) or records an alternative; Processing can be developed provisionally but not integrated |
| Voxel Architecture absent | PC-1 stays Provisional; formats cannot be Owner-Approved | RG-1; minimal PC-1 slice |
| Voxel-only types reaching P0 | rework of every layer | section 5 model is a PS-1 entry condition |
| Hidden second write path | authority and history bugs | section 6 enforcement |
| Many unassigned owners (WorldGen, NET, SRV, CLI, ENG) | stalls from PS-11 on | GOV-03 decisions with stage deadlines |
| Spike code reaching `master` | contradicts roadmap | GOV-04 baseline; PR rules |
| No physical devices | DG gates stall | DEV-01; device gates do not block PS-1…PS-13 |
| `panic = "abort"` across GDExtension | crashes | ADP-03 test before adoption |

## 14. Entry criteria for PS-1 (verifiable)

All of the following, each with evidence:
1. Decision-registry entries (owner-confirmed): ACR-001, ACR-003, ACR-004 part A, part D principle (OAM-10), D-R1, D-R2, D-R6, D-R8, D-K, D-T, D-C, D-0 (task GOV-01).
2. P0-alpha Owner-Approved; PC-1 registered Provisional with the mandatory fields (GOV-02).
3. Named owners for Core Agent, Integration Owner, Agent I (GOV-03); unassigned modules are not required for PS-1.
4. Docs-only baseline on `master` (GOV-04).
5. CI **specification** Owner-Approved (GOV-05) **and** CI **implementation** running with a workflow-run URL showing green on an empty skeleton and red on seeded failures (GOV-06); `archcheck` v0 (XT-01) and `cube-testkit` skeleton (TEST-01) merged.
6. This plan's ADG validated (report section on validation) and adopted.
Not required for PS-1: physical devices, Android emulator runs, the Geometry Processing Specification, any WorldGen/server/client owner.

## 15. PS-1 exit criteria (verifiable)

1. CORE-01…CORE-11 merged; `G1` conditions satisfied (ADG part D).
2. Contract tests pass for C-01, C-02, C-03, C-04, C-06, C-16, C-17, C-18, C-19 (P0-alpha) and for the PC-1 slice, **on the CI host** (desktop); no emulator or device run is part of this exit.
3. Property tests (atomicity, snapshot isolation, monotonic revisions, negative-coordinate symmetry) and loom tests of the commit gate pass; golden hashes reproducible.
4. Negative tests prove the mutation boundary (TEST-06 subset available at PS-1: compile-fail and commit-log tests in CORE-08/XT-02).
5. Public-API snapshot published (CORE-10) and enforced by CI.
6. Owner records: P0-alpha **Implemented** acknowledged; PC-1 **Implemented-Provisional**; no contract Frozen.
7. No dependency on `godot`, I/O, clocks or randomness in `cube-core` (archcheck).

## 16. Open items kept visible

Everything in section 3.3 (confirmation of AOD-01…24), the unassigned owners, O-1 (derived-data provider ownership), O-2 (modifier evaluation), Import/Export owner, the Geometry Processing Specification, the Voxel Architecture, GDD/TDD, the original AU-1.0, ACR decisions, the scope of gameplay/content/MMO beyond this plan, and the numeric performance thresholds (set from device data) remain open. See `architecture-blocker-decisions.md` and the report.
