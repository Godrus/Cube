# GOV-01 Decision Register (preparation)

**Status:** **Proposed.** This is a preparation document. It decides nothing and sets no status. Nothing here is an owner decision until the owner writes it into the decision registry with a confirmation source.
**Base:** repository HEAD `64e8a760cc01cfd344907f22cc93a889a6e614b1` (verified 2026-11-05: branch `docs/architecture-roadmap-v1`, working tree: only the owner's ` M .gitignore`). The claimed baseline `64e8a76` is confirmed as HEAD and as a docs-only commit (6 files, all under `docs/`).
**Where the 15 come from (verified):** `docs/production/ps-0-exit-checklist.md` section 2 (rows 1-15, all NOT DONE) = task `GOV-01` in `docs/production/agent-task-breakdown.md` row GOV-01 + `architecture-blocker-decisions.md` sections 1-2. The list in the task row and the checklist match item for item.
**Companions:** `GOV-01-TECHNICAL-RECOMMENDATIONS.md` (recommendations, architecture check, prioritisation), `ADG-D01-D02-ANALYSIS.md`, `ST-1-CANONICAL-STATE-RECOMMENDATION.md`, `PS-0-REMAINING-BLOCKERS.md`.

## 0. How a decision becomes effective (rule from the plan)

`production-architecture-plan.md` section 3.3: only an entry in the decision registry with a **confirmation source** counts: (a) an owner-signed commit or pull-request approval in the repository, or (b) a decision text written by the owner in the repository. **A chat instruction alone is not a confirmation source.** Agents may transcribe a chat instruction only as `recorded, awaiting confirmation`.
Registry path: the plan says `docs/decisions/decision-registry.md` (section 3.3); some later documents (including documents of the previous package) say `docs/production/decision-registry.md` or only `decision-registry.md`. See finding F-2 in `PS-0-REMAINING-BLOCKERS.md`; this register uses the plan's path.

Entry template (for the owner or for an agent quoting the owner):

```
### <ID>  <title>
Date: <YYYY-MM-DD>   Decision: <Approved | Amended | Rejected | Deferred>   Option: <label>
Text: <the owner's words, verbatim, or the amendment>
Confirmation source: <owner-signed commit hash | owner-written file+commit>
Supersedes / affects: <ids>
Recorded by: <role>   (an agent never sets Owner-Approved without the source)
```

## 1. Summary table

| # | ID | Title | Status now | Class (see Recommendations doc) | Owner choice needed? |
|---|---|---|---|---|---|
| 1 | ACR-001 | Foundation types live in Core | Proposed, no entry | Blocking PS-0 | confirm |
| 2 | ACR-002 | Ports in `cube-editing-api` (L2) | Proposed, no entry | Deferred (conditional) | confirm |
| 3 | ACR-003 | Composite data in Core, operations in `cube-composite` | Proposed, no entry | Deferred | confirm |
| 4 | ACR-004 part A | Core prerequisites for system commits | Proposed; ACR-004 as a whole **Pending** | Blocking PS-0 | confirm |
| 5 | OAM-10 / ACR-004 part D | Revision model for system changes | Recommendation only (R-A) | Blocking PS-0 | **choose** |
| 6 | OD-R1 | Revision token width | Proposed (spike has `u64`/`u32`) | Blocking PS-0 | confirm |
| 7 | OD-R2 | Commit algorithm | Proposed (spike: validate-then-apply under `&mut`) | Blocking PS-0 | **choose** |
| 8 | OD-R6 | RegionStamp | Proposed; hash algorithm unnamed | Blocking PS-0 | **choose** (algorithm) |
| 9 | D-R8 | OperationId format | Proposed (spike: `OperationId(u64)`) | Blocking PS-0 | confirm |
| 10 | D-K | Cell key, widths, brick edge | Proposed; becomes Provisional PC-1 | Blocking PS-1 | **choose** |
| 11 | D-T | Authoritative transform | Proposed; scope ambiguous vs. Mesh layers | Blocking PS-0 | **choose** (with amendment) |
| 12 | D-C | Channel model | Proposed; becomes Provisional PC-1 | Blocking PS-1 | **choose** |
| 13 | D-0 | P0 scope: P0-alpha + PC-1 | Proposed | Blocking PS-0 | **choose** |
| 14 | D-E | Toolchain, execution model, licence allow-list | Proposed; allow-list absent | Blocking PS-0 | **choose** |
| 15 | branch | Docs-only baseline option | Proposed; undecided | Blocking PS-0 | **choose** |

Totals: 15 found. Blocking PS-0: 11. Blocking PS-1: 2. Deferred: 2. Personal choice: 9; confirm-only: 6.

## 2. Decisions

Common fields for every item: **Status:** Proposed; no registry entry; not confirmed. **Documents to change after approval (always):** `docs/decisions/decision-registry.md` (new entry), `docs/PROJECT_STATUS.md` (5.5, 5.3), `docs/production/ps-0-exit-checklist.md` (row state); further documents are listed per item.

---

### 1. ACR-001 — Foundation types in Core

**Question:** Does Core own `RevisionToken` (C-16), `OperationId` (C-17), `DirtyRegion` (C-18) and `Result/Error` (C-19), although AU-1.0 section 23 lists them under Editing/Processing?
**Sources:** `architecture-blocker-decisions.md` section 1 (ACR-001); `api-contracts/core-contracts.md` C-16…C-19; `api-contracts/README.md` (C-16 "OD-R1…R8"); AU-1.0 (reconstructed copy, not byte-verified; original absent, so the text of section 23 cannot be confirmed: **missing data**).
**Why decide:** Core's commit produces revisions, dirty regions and operation ids and returns `Result`; P0-alpha (GOV-02) includes C-16…C-19.
**Options:**
* **A. Core owns them (as proposed).** + No upward dependency; commit is self-contained; already true in the spike (`cube-core` has `ids.rs`, `revision.rs`, `dirty.rs`, `error.rs`: verified). − Deviates from AU-1.0 section 23 wording.
* **B. New `cube-foundation` crate below Core.** + Keeps AU-1.0 grid literal; Core stays "geometry only". − Extra crate and archcheck edge; every crate depends on it; no technical gain, since only Core produces these values.
* (Keeping them in Editing/Processing is not offered: Core would have to depend upward, breaking one-directional dependencies.)
**Consequences:** A: none on code. B: new crate, ownership entry, renames in all contracts.
**Impact on stages:** PS-1 (CORE-06/07/08/09), PS-2 (CORE-16), everything that consumes tokens.
**Deferral consequence:** GOV-02 cannot approve C-16…C-19 (P0-alpha); G0 stays open.
**Documents to change:** `docs/api-contracts/README.md` (status/owner notes), `docs/production/module-boundaries.md` (A.3 Core row, if B), `architecture-blocker-decisions.md` (state).

### 2. ACR-002 — Orchestration reaches backends through L2 ports

**Question:** Are the Editing ports (`IEditingService`, `IGeometryBackend`, `ITransactionObserver`) defined in `cube-editing-api` (L2), with the backend registry in Orchestration?
**Sources:** `architecture-blocker-decisions.md` (ACR-002); `api-contracts/editing-contracts.md`; QA Pack (rejected the direct dependency, per the blocker text); `module-boundaries.md` Part B.
**Why:** prevents a cycle Orchestration ⇄ backends and lets backends compile without Orchestration (API-02 acceptance).
**Options:** **A.** ports in L2, registry in Orchestration (proposed). + one-directional graph; backends testable alone; matches layers. − one more crate. **B.** Orchestration depends directly on backends. + fewer crates. − cycle risk, test coupling, rejected in the QA Pack.
**Consequences:** A requires moving spike voxel logic from `Editor` into a voxel backend (VOX-01).
**Impact:** PS-3 API-02 freeze, PS-4, PS-5, archcheck v0 edge table (XT-01, see Recommendations 3.2).
**Deferral consequence:** none before PS-3 *unless* archcheck v0 encodes the L2 edges at PS-0 (then it blocks XT-01); plan text puts it in GOV-01.
**Documents to change:** `module-boundaries.md` (Part B), `docs/api-contracts/editing-contracts.md` (provider note), `architecture-blocker-decisions.md`.

### 3. ACR-003 — Composite data in Core, operations in Composite

**Question:** Do `CompositePart` (C-14) and `AssemblyGraph` (C-15) data types live in Core, with operations in `cube-composite`?
**Sources:** blocker decisions (ACR-003); AU-1.0 section 23 vs section 2 conflict (original absent: **missing data**); `core-contracts.md` C-14, C-15.
**Why:** Persistence must store composites; if the model lived in Composite, Persistence would depend on Composite.
**Options:** **A.** passive data in Core, operations in Composite (proposed). **B.** whole model in Composite (adds a Persistence→Composite edge). **C.** everything in Core (puts operation behaviour in Core, violating ownership).
**Consequences:** A: nothing now; C-14/C-15 deferred contracts anyway.
**Impact:** CORE-14 (PS-8), COMP-*, PERS-09, GOV-19.
**Deferral consequence:** none until PS-8 start.
**Documents to change:** `module-boundaries.md`, `core-contracts.md` ownership note, `agent-task-breakdown.md` (only if B or C).

### 4. ACR-004 part A — Core prerequisites for system commits

**Question:** Approve: `ActorKind` (`User`, `System`, `Recovery`) on `OperationId`; commit-primitive capability handles `UserCommit`/`SystemCommit`/`RecoveryCommit` and their port types; minimal Core primitives; the authoritative-path rule (exactly three callers write: Editing service, Materialiser, persistence loader)?
**Sources:** `architecture-blocker-decisions.md` section 2 (part A); `decisions/acr-004-owner-review.md` Part A/C; `api-contracts/system-commit-contracts.md` (SC-01, SC-02, SC-05, SC-10, Draft); AOD-20 (hybrid C3, Approved as recorded); production plan MB-1.
**Why:** AOD-20 says system generation takes part in commit/revision semantics but is not an ordinary Editing-API operation; Core must supply the boundary before WorldGen exists, and P0-alpha includes these amendments.
**Options:** **A. Approve.** **B. Amend** (e.g. one `SystemCommit` handle with an enum instead of two handles; or different `ActorKind` names). **C. Reject** — then WorldGen would have to write as an ordinary Editing-API client, which contradicts AOD-20 (so C is valid only together with a changed AOD-20).
**Pros/cons:** A: compile-time proof of who can mutate; − more types in Core. B: depends on what the owner wants to change. C: simpler Core; − violates AOD-20, makes system writes undoable by default.
**Consequences:** compile-fail tests (TEST-06), commit log, archcheck rule XT-02.
**Impact:** CORE-07/08/09, TEST-06, XT-02, later WGEN/PERS/LEG.
**Deferral consequence:** P0-alpha cannot be Owner-Approved; PS-1 cannot start.
**Open items that stay open (not decided here):** ABI/signature of the handles, whether `Evict` is a commit for revision purposes, audit-log retention, `ChangeType` additions.
**Documents to change:** `api-contracts/system-commit-contracts.md` (status), `core-contracts.md` (C-17 extension), `acr-004-owner-review.md` (Part A state), `owner-architecture-decisions-v1.md` (ACR-004 record), `module-boundaries.md`.

### 5. OAM-10 / ACR-004 part D — Revision model for system changes

**Question:** How do system (generation) changes interact with revisions?
**Sources:** `decisions/owner-approval-matrix-v1.md` OAM-10 (lines ~355-370); `acr-004-owner-review.md`; `api-contracts/revision-and-snapshot-semantics.md`.
**Why:** C-16 (P0-alpha) must define what a commit increments; part D is the only part that touches the P0 revision contract.
**Options (labels as in OAM-10; note: the label "R-A" is also used by OAM-05 for the renderer `faces`, so registry entries must write `OAM-10/R-A`):**
* **R-A** same asset/layer/cell revision scopes for system commits, `ActorKind::System` marks the origin. + no new token kind; uniform undo/conflict rules; least code; journal records only user commits. − asset/layer revision advances during streaming (one per streamed chunk in the spike), so a coarse revision no longer means "a user changed something"; user edits must be validated with brick/region stamps, not asset-level tokens.
* **R-B** separate generation epoch. + user vs terrain updates distinguishable by token. − new revision kind, changes C-16 comparison rules, more state to persist.
* **R-C** separate revision domains for generated base and user overrides. + strongest separation. − largest change to C-16, Materialiser needs its own handling, couples to the C1/C2 choice (OAM-09, deferred).
**Consequences:** R-A makes OD-R6 (region stamp) mandatory, because asset-level tokens would give spurious conflicts.
**Impact:** CORE-06, PERS recovery (ACR-004 C), WGEN materialiser, server replication filter.
**Deferral consequence:** C-16 text cannot be approved; PS-1 CORE-06 cannot start.
**Documents to change:** `revision-and-snapshot-semantics.md`, `core-contracts.md` C-16, `system-commit-contracts.md` SC-04, `module-boundaries.md` (A.3 core row mentions R-A as Proposed), `acr-004-owner-review.md`.

### 6. OD-R1 — Revision token width

**Question:** `RevisionToken` value `u64`, epoch `u32`; overflow is a critical error and never wraps; widths are named, versioned constants?
**Sources:** blocker decisions (OD-R1); `crates/cube-core/src/revision.rs` (verified: `epoch: u32`, `value: u64`, comment "OD-R1, OD-R2").
**Why:** fixes the persisted and wire size; needed before C-16.
**Options:** **A.** `u64`/`u32`, overflow = error (proposed). **B.** `u32` value with rollover. + smaller. − rollover breaks monotonic comparison and replay validation.
**Consequences:** A: persisted schema carries named width constants (serialized form is separate from the in-memory layout).
**Impact:** CORE-05/06, PERS, NET.
**Deferral consequence:** C-16 cannot be approved.
**Documents to change:** `core-contracts.md` C-16, `revision-and-snapshot-semantics.md`.

### 7. OD-R2 — Commit algorithm

**Question:** Which concurrency shape implements atomic commit?
**Sources:** blocker decisions (OD-R2); `crates/cube-core/src/asset.rs` (verified: `commit(&mut self, WriteSet)`, validate-then-apply); `revision-and-snapshot-semantics.md` section 4.1.
**Why:** it fixes the shared-asset API (`SharedAsset`), the snapshot model and loom tests; needed before C-16/C-01.
**Options:** **A.** immutable `Arc` state, per-asset commit gate, lock-free snapshot readers; finer-grained gating later behind the same API (proposed). + readers never block; matches immutable snapshots; validate-then-apply stays one atomic step. − copy-on-write cost per commit (mitigated by 16³ COW bricks); needs loom tests. **B.** cell-level compare-and-swap. + maximal write concurrency. − multi-brick atomic transactions become hard; validate-then-apply across bricks is no longer one step; much harder to test. **C.** keep `&mut` single-writer (spike). + simplest. − not shareable between server threads without an external lock; this is what A wraps.
**Consequences:** A requires `SharedAsset`, loom dev-dependency (D-E).
**Impact:** CORE-01, CORE-06, PROC (snapshots), ORCH, server.
**Deferral consequence:** C-01 and C-16 cannot be approved.
**Documents to change:** `core-contracts.md` C-01/C-16, `revision-and-snapshot-semantics.md` 4.1.

### 8. OD-R6 — RegionStamp

**Question:** How is "the region is unchanged" summarised?
**Sources:** blocker decisions (OD-R6); `revision-and-snapshot-semantics.md` section 2; not implemented in the spike (snapshot compares per cell, per the blocker text).
**Why:** under R-A (item 5) asset-level tokens are useless for conflict detection; region/brick stamps carry user-edit validation.
**Options:** **A.** stable 128-bit hash over sorted `(CellKey, revision)` plus count; snapshots keep exact per-cell revisions (proposed). + O(1) comparison, no false positives beyond hash collisions. − the hash algorithm becomes a contract. **B.** materialised set. + exact. − memory O(cells). **C.** coarse per-layer stamps. + trivial. − false conflicts, defeats region validation.
**Missing data:** the hash algorithm is not named in any source. The owner must either name it or delegate to a later registry entry before C-16 approval.
**Impact:** CORE-05, PROC snapshots, NET, client.
**Deferral consequence:** C-16 incomplete.
**Documents to change:** `core-contracts.md` C-16, `revision-and-snapshot-semantics.md`.

### 9. D-R8 — OperationId format

**Question:** `OperationId = (actor scope u64, sequence u64)` with `ActorKind` separate; `Recovery` keeps original ids; the format carries a schema version?
**Sources:** blocker decisions (D-R8; some contract tables call it OD-R8); `crates/cube-core/src/ids.rs` (verified `OperationId(pub u64)`); `core-contracts.md` C-17.
**Why:** idempotent retry, reconnect, journal replay need stable ids; needed before C-17.
**Options:** **A.** composite id (proposed). + deterministic, ordered per actor, dedupe by (scope, sequence). − two words. **B.** 128-bit random. + no coordination. − unordered, no per-actor replay ordering, larger. **C.** client-supplied. + client retries trivially. − untrusted client input, must be remapped (AOD: client is not the source of truth).
**Missing data:** dedupe window/retention policy for idempotent retry is not specified.
**Impact:** CORE-07, ORCH, HIST, PERS journal, NET.
**Deferral consequence:** C-17 cannot be approved.
**Documents to change:** `core-contracts.md` C-17, `editing-contracts.md` E-01 note.

### 10. D-K — Cell key, widths, brick edge

**Question:** `CellKey{layer, level: i8, coord: i32×3}`, brick edge 16 as a named constant?
**Sources:** blocker decisions (D-K); `crates/cube-core/src/coord.rs` (verified: `BRICK_SIZE: i32 = 16`, coordinates `i32`); plan section 4.3 PC-1 basis BL-1; Voxel Architecture **absent**.
**Why:** every crate and every persisted byte depends on it; it is the assumption of Provisional PC-1.
**Options:** **A.** as proposed (i8 level, i32 coordinates, edge 16). + matches the spike (all tests and benchmarks) and the 4096-cell brick; i32 gives about ±2.1×10⁹ cells per axis, explicit limit. − "practically endless" world is bounded per axis by that range (declare it); fixed edge. **B.** `i64` coordinates. + no practical bound. − double key size, memory and wire size, unmeasured. **C.** edge 8 or 32. + different COW/cache trade-off. − no data; needs Voxel Architecture.
**Missing data:** Voxel Architecture; no benchmark compares edges.
**Consequences:** recorded as Provisional with revalidation gate RG-1; formats do not depend on it until RG-1 (GOV-10).
**Impact:** C-07, CORE-03, PS-5 backends, PS-10 format.
**Deferral consequence:** CORE-01/03 (key/brick work) cannot start; PC-1 register entry has no basis.
**Documents to change:** `core-contracts.md` C-07, `production-architecture-plan.md` 4.3 (PC-1 register), `api-contracts/README.md`.

### 11. D-T — Authoritative transform

**Question:** Is the authoritative `GeometryTransform` an integer translation + one of the 24 axis-aligned rotations + power-of-two scale, with no floating point in authoritative or persisted state?
**Sources:** blocker decisions (D-T); `core-contracts.md` C-04 (open "f32/f64, epsilon"); spike `voxel_size_m: f32` (verified in `asset.rs`); integration plan section 3 archcheck rule "no floating point in authoritative or world-defining code per D-T"; plan section on closed `LayerData::{Voxel,Sdf,Mesh}`.
**Why:** determinism across platforms/servers (server-authoritative, cross-play, replay); C-04 is P0-alpha.
**Options:** **A.** integer transform as proposed. + bit-exact on all platforms; no epsilon policy; hashes stable. − arbitrary rotation/scale of imported content must be baked. **B.** `f64` with an epsilon policy. + general transforms. − non-determinism risk across platforms, hash instability, epsilon rules in every comparison. **C. (amendment I propose to put to the owner)** A, with a **scope clause**: D-T governs transforms and coordinates only; the representation of SDF samples and mesh vertices inside a layer is decided separately (with RG-1) and is not covered by "no floating point".
**Why C is raised:** as written, "no floating point in authoritative or persisted state" and the archcheck rule can be read to forbid float mesh vertices or SDF values, but Mesh layers are a closed representation in Core. The documents do not say how mesh data is stored: **missing data / ambiguity**.
**Impact:** C-04, CORE-02, archcheck XT-01/XT-02 rule text, backends (SDF/Mesh), client adapter.
**Deferral consequence:** C-04 cannot be approved (P0-alpha).
**Documents to change:** `core-contracts.md` C-04, `integration-and-qa-plan.md` section 3 (archcheck rule wording), `production-architecture-plan.md`.

### 12. D-C — Channel model

**Question:** Built-in channel set: `Material(u16)`, quantised `Density`, `Attribute`, with Constant/Dense encodings and a per-layer quantisation scale?
**Sources:** blocker decisions (D-C); `core-contracts.md` C-08, C-09 ("layout, index order, compression"; "attribute registry, widths" open); spike `BrickData::{Constant(u16),Dense}` (verified); Voxel Architecture **absent**.
**Why:** minimal slice needed to store any cell; assumption of PC-1.
**Options:** **A.** as proposed. **B.** fixed per-kind layout. + simplest. − no extensibility for attributes. **C.** float density. + simple SDF. − conflicts with D-T intent if read broadly, breaks hash stability.
**Missing data:** quantisation widths and the attribute registry are unspecified.
**Impact:** C-08/C-09, PS-5 SDF/mesh backends, PS-10 format.
**Deferral consequence:** no basis for the PC-1 register entry; storage tasks cannot start.
**Documents to change:** `core-contracts.md` C-08/C-09, PC-1 register.

### 13. D-0 — P0 scope: P0-alpha + PC-1

**Question:** Is PS-1 closed on P0-alpha (C-01, C-02, C-03, C-04, C-06, C-16, C-17, C-18, C-19, including the ACR-004 part A amendments SC-01/02/05/10) Owner-Approved, with C-07, C-08, C-09 as Provisional PC-1, and C-05, C-10…C-15 deferred to the stages that consume them?
**Sources:** blocker decisions (D-0); plan sections 4.1-4.4; QA Pack (requires P0 approved before the Core stage); AOD-18 (contracts stay Draft until separately approved), AOD-24 (temporary baseline allowed, never substitutes for the missing sources).
**Why:** Voxel Architecture and Geometry Processing Spec are absent, so full P0 cannot be approved honestly.
**Options:** **A.** P0-alpha + PC-1 (proposed). + PS-1 can start; honest about Provisional. − PC-1 may change once at RG-1 (rework budget). **B.** wait for all sources. + no rework. − blocks the project indefinitely (no date for the sources). **C.** approve full P0 on baseline entries. − contradicts AOD-24.
**Impact:** all of PS-1, the Provisional register, GOV-02, INT-01.
**Deferral consequence:** nothing can be approved or started.
**Documents to change:** `api-contracts/README.md` (status column), `production-architecture-plan.md` 4.x, `module-boundaries.md` Part D (G1 text), `ps-0-exit-checklist.md`.

### 14. D-E — Toolchain, execution model, dependency licences

**Question:** Pinned stable Rust and edition; no async runtime in Core/Processing/Backends (a std-thread `Executor`); dev-dependencies `proptest`, `loom`, `trybuild`, `criterion`; `cargo deny` with a licence allow-list; `catch_unwind` at the FFI; test `panic = "abort"` before adopting it?
**Sources:** blocker decisions (D-E); `integration-and-qa-plan.md` section 3 (required checks); AOD-23 (licence policy); root `Cargo.toml` (verified: edition 2021, `panic = "abort"` in `[profile.release]`, `license = "UNLICENSED"`, `publish = false`); local toolchain observed `cargo 1.97.1` (not a decision).
**Why:** GOV-05 (CI specification) and GOV-06 (CI) need pinned tools, the licence policy and test dependencies.
**Options:** **A.** as proposed. + deterministic, small lower layers. − Executor must be written. **B.** tokio/rayon allowed in lower layers. + libraries. − async leaks into the pure core, harder determinism and loom testing.
**Internal tension to resolve:** `catch_unwind` at the FFI has no effect under `panic = "abort"` (spike release profile); the decision must say which policy applies.
**Missing data (the owner must supply):** the actual Rust version to pin; the **licence allow-list itself** (the plan says it is the owner's decision); the licence of Cube's own code (currently `UNLICENSED`); the CI host/platform (GitHub Actions is implied by "required status checks" but not stated).
**Impact:** GOV-05, GOV-06, XT-01, every crate.
**Deferral consequence:** CI cannot be specified; G0 open.
**Documents to change:** `integration-and-qa-plan.md` section 3, `rust-toolchain.toml`/`deny.toml` (created later by GOV-06), `module-boundaries.md` (execution model note).

### 15. Branch option for the docs-only baseline

**Question:** How does `master` receive a docs-only baseline?
**Sources:** `integration-and-qa-plan.md` section 1; GOV-04 (acceptance: "no file of the spike code list is on master"); roadmap-v1 section 7 (spike code not for `master`).
**Facts (verified):** `master` has 13 files; the docs branch is 35 commits ahead and changes 268 files; the docs branch contains the spike crates, `godot-client/`, `scripts/`, `docs/spike/` (170 files). Production crate names (`crates/cube-core`, ...) are the same as the spike crate names, so mixing them would collide.
**Options:** **A.** a new docs-only branch from `master` with the docs commits (links into `docs/spike/` removed, or spike reports added by a separate docs commit). + clean base; spike stays on its own branches. − the docs commits must be re-created (cherry-pick of path-limited changes), so history differs from this branch. **B.** path-limited PRs. − GitHub PRs carry the whole branch diff; to limit paths you create separate branches that contain only those paths, which is option A by another mechanism. **C.** accept spike code as an explicit experimental directory. + no re-creation. − contradicts roadmap-v1 section 7 and the "spike never merges" rule; collides with production crate paths.
**Impact:** GOV-04, GOV-06, every production PR base.
**Deferral consequence:** no production base; GOV-04/06 blocked.
**Documents to change:** `integration-and-qa-plan.md` section 1 (state), `PROJECT_STATUS.md` 5.2.

---

## 3. Decisions adjacent to the 15 that G0 also needs (not in the checklist)

| ID | What | Why it matters | Source |
|---|---|---|---|
| X-1 | Countersignature of AOD-01…24 (and OAM-01/02/03/07/08) | plan 3.3 says "decision GOV-01 covers this"; the checklist's 15 rows do not list it | plan 3.3; blocker doc D-REG |
| X-2 | D-REG: adopt the status model (Proposed, Provisional, Owner-Approved, Implemented, Frozen, Rejected, Superseded) and the registry | GOV-02 "adopt the Contract Status Model"; no row in the 15 | blocker doc D-REG |
| X-3 | GOV-03 owners (WorldGen, Networking Adapter, Server Runtime, Client Runtime, Engine Adapter, Import/Export, O-1, O-2), or deferral in writing | G0 member; "defer each in writing" is allowed | agent-task-breakdown GOV-03 |
| X-4 | Owner approval of the CI specification (GOV-05) | G0 member; also ST-1 and ST-2 from the status package | integration plan section 3 |

These are listed because GOV-01…GOV-03 and GOV-05 are G0 members; the owner may fold X-1/X-2 into GOV-01.
