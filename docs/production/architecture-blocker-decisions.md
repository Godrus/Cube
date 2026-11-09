# Cube Architecture Blocker Decisions v2 (proposal for the owner)

**Status:** **Proposed**. Revision v2.1. Nothing here is a decision and nothing is represented as approved. Each decision has: 1 Problem, 2 Current implementation, 3 Proposed solution, 4 Alternative, 5 Compatibility impact, 6 Affected modules, 7 Migration strategy, 8 Required owner approval. The owner approves, amends or rejects each item in a **decision-registry entry with a confirmation source** (`production-architecture-plan.md` section 3.3). ACR-001…004 stay `Pending`. Task ids and stages refer to the ADG (`module-boundaries.md`).
**Differences from v1:** D-0 is rewritten (no contradiction with the PS-1 exit); ACR-004 is split into five independent parts with owners, contracts, dependencies, stages and acceptance; the registry/status model is a decision; revalidation gates RG-1/RG-2 added; D-R8 and D-P extended.

## 1. Decisions needed before PS-1 (task GOV-01 / GOV-02)

### D-0: P0 scope, P0-alpha and Provisional Contracts
1. **Problem:** the QA Pack requires P0 approved before the Core stage, but Voxel Architecture (constrains C-07…C-12) and the Geometry Processing Specification (PR-01…PR-08) are absent; AOD-24 forbids final approval on a baseline.
2. **Current:** all 19 Core contracts are Proposed (Draft); baseline entries BL-1…BL-3 are candidates.
3. **Proposed:** formal split: **Full P0** (C-01…C-19), **P0-alpha** = C-01, C-02, C-03, C-04, C-06, C-16, C-17, C-18, C-19 (core slices), **Provisional PC-1** = C-07, C-08, C-09 (minimum slice), **deferred** C-05, C-10, C-11, C-12, C-13, C-14, C-15 (separate Core tasks in their consumer stages). PS-1 closes only on an Owner-Approved P0-alpha; PC-1 is Provisional with the five mandatory fields and revalidation gate RG-1; Provisional never becomes Frozen automatically.
4. **Alternative:** wait for all sources (safest, blocks PS-1); or approve full P0 on baseline entries (contradicts AOD-24).
5. **Compatibility:** PC-1 may change once at RG-1; tests carry a `provisional` tag; formats do not depend on PC-1 until RG-1 (GOV-10).
6. **Affected:** Core, then all modules; Persistence format approval.
7. **Migration:** at RG-1 diff PC-1 against Voxel Architecture; budgeted rework task in the stage that revalidates.
8. **Approval:** required: the P0-alpha list, the PC-1 register entry and the rule that PS-1 exit means P0-alpha only.

### D-REG: contract status model and decision registry
1. **Problem:** documents mix `Draft`, `Proposed`, `Approved`; agent-written records (AOD-01…24) look like owner decisions.
2. **Current:** statuses defined in `docs/api-contracts/README.md`; AOD records written by an agent from chat.
3. **Proposed:** statuses Proposed, Provisional, Owner-Approved, Implemented, Frozen, Rejected, Superseded with assigners and evidence (architecture plan 3.1); a decision registry whose entries need an owner-signed repository confirmation; AOD-01…24 stay `recorded, awaiting confirmation` until countersigned.
4. **Alternative:** keep the legacy words and a note per document (weaker, ambiguous).
5. **Compatibility:** renames only; no contract text changes.
6. **Affected:** all documents.
7. **Migration:** update the contracts README status column and add the registry (task GOV-02) on the owner's record.
8. **Approval:** required.

### ACR-001, ACR-002, ACR-003 (unchanged proposals from v1, restated)
* **ACR-001** (foundation types in Core): 1 Problem: Core produces revisions/dirty/ids but AU-1.0 §23 lists them under Editing/Processing. 2 Current: already in `cube-core` (fact; implemented before decision). 3 Proposed: approve Core ownership of C-16…C-19. 4 Alternative: a `cube-foundation` crate. 5 Impact: none on code. 6 Affected: Core, Processing, API, Persistence, Net. 7 Migration: none. 8 Approval: required before P0-alpha.
* **ACR-002** (orchestration reaches backends via L2 ports): 1 cycle risk and test coupling. 2 no code yet; the spike `Editor` holds voxel logic. 3 approve: ports in `cube-editing-api`, registry in Orchestration. 4 direct dependency (rejected in the QA Pack). 5 none. 6 API, Orchestration, backends, tools, history. 7 move voxel logic into `cube-backend-voxel` (VOX-01). 8 required before API-02 freeze.
* **ACR-003** (composite data in Core, operations in Composite): 1 AU-1.0 §23 vs §2 conflict. 2 nothing implemented. 3 approve passive data in Core (CORE-14), operations in `cube-composite`. 4 whole model in Composite (adds a Persistence edge) or all in Core (violates ownership). 5 none. 6 Core, Composite, Persistence (PERS-09). 7 none. 8 required before CORE-14 and PS-8.

### OD-R1 / OD-R2 / OD-R6 / D-R8 (Core token and commit surface)
* **OD-R1** token width: 1 Problem: overflow, wire and persisted size. 2 `u64` value, `u32` epoch. 3 keep; overflow is a critical error, never wraps; widths are named, versioned constants. 4 `u32` value with rollover. 5 fixes persisted schema. 6 Core, Persistence, Net. 7 none. 8 required before C-16.
* **OD-R2** commit algorithm: 1 concurrency shape. 2 validate-then-apply under `&mut` (correct, single-threaded access). 3 immutable `Arc` state, per-asset commit gate, lock-free snapshots (hybrid fine-grained later behind the same API). 4 cell-level CAS. 5 semantics unchanged. 6 Core, Processing, Orchestration. 7 wrap the spike algorithm in `SharedAsset`; port tests; add loom. 8 required before C-16/C-01.
* **OD-R6** RegionStamp: 1 summarise many cell revisions. 2 not implemented; snapshot compares per cell. 3 stable 128-bit hash over sorted `(CellKey, revision)` plus count; snapshots keep exact per-cell revisions. 4 materialised set; coarse per-layer stamps. 5 hash algorithm becomes contract. 6 Core, Processing, Net, Client. 7 new code (CORE-05). 8 required before C-16.
* **D-R8** OperationId format: 1 idempotent retry, reconnect, journal replay. 2 `OperationId(u64)`. 3 composite id `(actor scope u64, sequence u64)`; `ActorKind` separate; `Recovery` keeps original ids; the format carries a schema version. 4 128-bit random; client-supplied ids. 5 persisted in the journal. 6 Core, Orchestration, History, Persistence, Net. 7 none. 8 required before C-17.

### D-K, D-T, D-C (Core data shape)
* **D-K** key, widths, brick edge: Proposed `CellKey{layer, level: i8, coord: i32x3}`, edge 16 as a named constant; alternative i64 coordinates or edge 8/32; affects every crate and persisted data; approval before C-07 (which is Provisional until RG-1 anyway).
* **D-T** authoritative transform: Proposed integer translation + 24 axis-aligned rotations + power-of-two scale; no floating point in authoritative or persisted state; alternative f64 with an epsilon policy; replaces the spike's `voxel_size_m: f32`; approval before C-04.
* **D-C** channel model: Proposed built-in channel set (`Material(u16)`, quantised `Density`, `Attribute`) with Constant/Dense encodings and a per-layer quantisation scale; alternative fixed per-kind layout or float density; approval before C-08/C-09 (Provisional).

### D-E: toolchain, execution model, dependency licences
Proposed: pinned stable Rust and edition; no async runtime in Core/Processing/Backends (std-thread `Executor`); dev-dependencies `proptest`, `loom`, `trybuild`, `criterion`; `cargo deny` with a licence allow-list; `catch_unwind` at the FFI; test `panic = "abort"` before adopting it. Alternative: tokio/rayon in lower layers. The licence allow-list is the owner's decision (licence policy AOD-23 is recorded; Cube's own licence is open).

## 2. ACR-004 split into five independent parts

ACR-004 as a whole stays **Pending**. The architectural decision AOD-20 (hybrid C3) is recorded and is not reopened by any part. Each part can be decided independently; a rejected part does not invalidate the others, but the dependencies below apply.

| Part | Content | Owner | Contracts | Depends on (ADG) | Stage | Acceptance criteria |
|---|---|---|---|---|---|---|
| **A. Core prerequisites** | `ActorKind`; `OperationId` extension; system-commit boundary (`SystemCommit`/`RecoveryCommit` handles and port types); minimal Core primitives; authoritative-path rules | Core Agent (Integration Owner reviews) | C-17 extension, C-16/C-18 usage, commit primitive; SC-01, SC-02, SC-05, SC-10 (Proposed) | decision GOV-01; CORE-07, CORE-08, CORE-09 | PS-1 | compile-fail tests show an unauthorised crate cannot construct a handle; commit-log completeness test; system and recovery commits idempotent; XT-02 and TEST-06 pass; **no WorldGen, streaming or recovery code in PS-1** |
| **B. WorldGen integration** | materialisation, eviction, streaming, interaction with Core | WorldGen owner (UNASSIGNED) | WG-01…WG-10, SC-05/SC-10 | GOV-11, CORE-09, WGEN-01…04 | PS-11 (11B) | only the Materialiser writes generated data; eviction never drops undurable user data; no stale result applied; ITG-02 passes |
| **C. Persistence recovery** | recovery, replay, interaction with revision semantics, operation journal | Agent H | WP-04, WP-05, WP-07, WP-08; P-03/P-04 reconciliation | GOV-11, PERS-04, PERS-05, PERS-06, CORE-09, API-04 | PS-10 | AOD-21 rules verified by the crash matrix; recovery checks hash, epoch, revision, generator version; `Recovery` actor only through the port; replay idempotent |
| **D. Revision semantics** | rules for creation and propagation of system changes; consistency with commit and recovery | decision: project owner; implementation: Core Agent (commit), Agent H (recovery), WorldGen owner (materialiser) | C-16, SC-04; OAM-10 | decision in GOV-01 (principle); CORE-06 | decided in PS-0; consequences in PS-1, PS-10, PS-11 | Proposed model R-A: same asset/layer/cell scopes, `ActorKind::System` in results, no separate generation epoch without demonstrated need; user edits validated with cell/region stamps; revisions monotonic across restart and recovery |
| **E. Removal of legacy direct writes** | `WorldStreamer` direct commits and `set_voxel`; `WorldBridge` `edit_voxel`, `fill_box`, undo/redo, `apply_batch`; other detected bypasses (`Editor::replace_asset` reviewed) | WorldGen owner (streamer), Engine Adapter owner (bridge) - UNASSIGNED | SC-xx, E-xx | GOV-11, LEG-01 (after WGEN-03/04, PERS-08, ITG-02), LEG-02 (after ORCH-05, HIST-02, CLI-01, ADP-01, TEST-06) | PS-11, PS-12 | archcheck finds no commit/handle use and no history pattern in worldgen and adapter crates; regression suites pass without the removed paths |

Open questions that stay open and visible: ABI of the handles (signature), whether `Evict` is a commit for revision purposes, audit-log retention, `ChangeType` additions, the owner of each unassigned party.

## 3. Other decisions (restated from v1 with unchanged proposals)

* **OD-R4** snapshot caps: Proposed per-snapshot budget, per-asset global budget and maximum age after which tasks are cancelled; numeric defaults from benchmarks; needed before PR-01 approval. Alternative: copy-on-capture.
* **OD-R5** undo conflicts: Proposed phase 1 reject with typed conflict, phase 2 optional cell-level partial undo behind an explicit policy; the delta records cell-level pre/post values. Alternative: always partial or rebasing.
* **OAM-09** C1 or C2: Proposed C1 (one logical layer, cell-level overrides recorded in the journal), C2 deferred behind a spike; needed before the journal record design and WorldGen/Persistence tasks (GOV-11).
* **D-P** persistence items left open by AOD-21: formats, journal granularity, compaction trigger and retention, per-world generator-version policy; Proposed: run SPK-PERS first, approve the format only after PERS-05 and RG-1 (GOV-10).
* **D-NET** transport: run SPK-NET; no transport chosen (GOV-12 after the spike).
* **D-OWN** owners for WorldGen, Networking Adapter, Server Runtime, Client Runtime, Engine Adapter, Import/Export, Derived Data Providers (O-1), Modifier Evaluation (O-2): not assigned in this revision; GOV-03.

## 4. Decision-to-task map

| Decision | Decided by (task) | Blocks (examples) |
|---|---|---|
| D-0, D-REG, ACR-001, ACR-002, ACR-003, OD-R1/R2/R6, D-R8, D-K, D-T, D-C, D-E, the branch option, ACR-004 A and D principle | GOV-01, GOV-02 | G0, then CORE-01…CORE-11 |
| ACR-004 B, C, E; OAM-09 | GOV-11 | WGEN-03, PERS-06, LEG-01, LEG-02 |
| PR-01…PR-08 approval | GOV-07 (after RG-2) | G2, every production merge of PS-4 and later |
| E-02…E-12 approval | GOV-09 | G3 |
| Persistence format | GOV-10 | G10 |
| Transport | GOV-12 | NET-04 |
| Renderer default | GOV-13 (after DG-1) | none of PS-1…PS-13 |
| Owners | GOV-03 | every UNASSIGNED task |
| OD-R4 snapshot caps | GOV-24 | PROC-06 |
| OD-R5 undo policy | GOV-25 | HIST-02 |
| C-05 | GOV-16 | CORE-15 |
| RG-1 revalidation record | GOV-08 | GOV-15, GOV-17, GOV-18, GOV-10 |
| C-10 | GOV-17 | CORE-12 |
| C-11, C-12 | GOV-18 | CORE-13 |
| C-13, C-14, C-15 | GOV-19 | CORE-14 |
| tool-level contracts | GOV-20 | SCU-02, BOO-01 |
| composite-operation and resample/refine contracts | GOV-23 | COMP-01, RES-01 |
| WG-01…WG-10 | GOV-21 | WGEN-02, WGEN-03 |
| SR-01…SR-10, protocol types | GOV-22 | NET-02, SRV-01 |
| Freeze of PC-1 | GOV-15 (after GOV-08) | release gate G14 |

## 5. Decisions registered (2026-11-09)

All items of section 1 and the part A / part D principle of section 2 are decided in `docs/decisions/decision-registry.md` (engineering decisions under the owner's delegation, DEC-001; each may be amended by the owner): ACR-001 DEC-055, ACR-002 DEC-056, ACR-003 DEC-057, ACR-004 A DEC-058, OAM-10/part D DEC-059, OD-R1 DEC-060, OD-R2 DEC-061, OD-R6 DEC-062, D-R8 DEC-063, D-K DEC-064, D-T DEC-065, D-C DEC-066, D-0 DEC-067, D-E DEC-068 (project licence: DEFERRED, DEC-070), branch option DEC-069. This section records the pointer only; the registry entries hold the exact decisions.
