# GOV-01 Decision Register (preparation)

**Status:** **Proposed.** This is a preparation document. It decides nothing and sets no status. Nothing here is an owner decision until the owner writes it into the decision registry with a confirmation source.
**Revision:** corrective pass (CR-01...CR-10), see `GOV-01-CORRECTIVE-REVIEW.md`. First version: commit `74566cd`.
**Base:** branch `docs/architecture-roadmap-v1` (the requested name `architecure_roadmap` does not exist; this is the only branch containing `74566cd`, and its upstream `origin/docs/architecture-roadmap-v1` also contains it). Source HEAD of this pass: `74566cd975df656282d7844e98cea79a3329a18b` (verified; working tree before the pass: only the owner's ` M .gitignore`).
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

## 0.1 Glossary (removes a terminology ambiguity found in the first version)

* **Voxel position (`VoxelPos`)**: integer position of one voxel in a layer at a given level. Its integer width is **not specified in any source** (CORE-02 lists the type; the spike uses `i32`).
* **Brick**: a 16x16x16 block of voxels (edge 16 proposed, D-K). **`BrickCoord`**: integer position of a brick, `floor_div(voxel, 16)` (spike: Euclidean division, verified in `crates/cube-core/src/coord.rs`).
* **`CellKey` = `BrickKey`**: production plan 5.3 is titled "CellKey (BrickKey)": `CellKey { layer, level: i8, coord: [i32; 3] }` where `coord` is the **brick** coordinate. Elsewhere in the plan and in this register, "cell" in "per-cell revision" means **brick**, not voxel. Revisions exist per asset, layer and **brick** (`revision-and-snapshot-semantics.md` section 1 table); there is no per-voxel revision.
* **`RegionStamp`**: derived, not stored; the contract text defines it as the set of `(BrickKey, BrickRevision)` for bricks intersecting a region (`revision-and-snapshot-semantics.md`, row "Region Revision").

## 0.2 Authority: who decides what (CR-01)

**Rule used throughout this package.** The **Project Owner** decides and approves: architecture decisions, ACRs, contract approval states (Provisional, Owner-Approved, Rejected, Superseded), formats, licences, assignment of owners, scope. The **Integration Owner** (an agent role, "Main Agent / Integration Owner") prepares, reviews, checks conformance and consistency, maintains the registry and the ADG, records a decision *only on the Project Owner's entry*, and gives *engineering* acceptance of modules. The Integration Owner never approves a contract or a decision on the Project Owner's behalf. This is the wording of the production plan (3.1: `Owner-Approved` is assigned by the "project owner only"; `integration-and-qa-plan.md` section 2: the Integration Owner "does not approve contracts or ACRs") and of AOD-18.

**Four kinds of authority are kept apart (corrected after the post-review: the first version mixed them).**

| Kind | Holder | Sources |
|---|---|---|
| Project Owner authority: decisions, ACRs, contract approval states, formats, licences, owner assignment, scope | Project Owner | AOD-18 (`owner-architecture-decisions-v1.md` line 28); plan 3.1 and 3.3 |
| Contract approval authority (setting a contract to Owner-Approved) | Project Owner per AOD-18 and plan 3.1; **Integration Owner per some older documents** (see conflicts below) | below |
| Module acceptance authority (a module or task is *Accepted*; engineering) | Integration Owner. **Not a conflict**: the plan keeps it (`agent-task-breakdown.md`: "Acceptance authority: Integration Owner") | Agent Development Specification line 507 ("Only Integration Owner can mark a module Accepted") and line 520 (Definition of Done: "Integration Owner approves acceptance"); Geometry Editing Specification lines 846-860 (module acceptance conditions; item 12 "Integration Owner approves") |
| Integration Owner responsibilities | Integration Owner | Agent Development Specification 3.1 (lines 58-71): architecture consistency, shared API contracts, dependency graph, integration order, Pull Request approval, release acceptance; owns the composition root and the dependency matrix |

**Genuinely supported conflicts (these are the only ones kept; the ruling is the owner's and is folded into X-2):**

| Document (line) | Says | Why it conflicts | Strength |
|---|---|---|---|
| `specifications/cube-agent-development-specification-v1.0.md` line 69 | "Only the Integration Owner may approve changes to shared public contracts." | The sentence is about *changes* to shared contracts; AOD-18 is about the *approval status* of contracts. It conflicts only if "changes" includes first approval; plan 3.1 does not say who approves a text change to an Owner-Approved contract (a change to a Frozen one needs an ACR and the owner) | **partial / ambiguous** |
| `agent-tasks/geometry-editing-agent-task-pack-v1.0.md` line 6 and line 710 | "Contract authority: Integration Owner"; "Integration Owner approves contracts by tier" | Contract *approval status* is assigned to the Integration Owner | **direct** |
| `decisions/audit-resolution-report.md` lines 211 and 244 | blocker B-4 "Integration Owner approval"; step 4 "Integration Owner: approve P0, then P1 contracts; set status to Approved and version to 1.0.0" | Same; the report is a dated historical document that predates the owner's AOD records of 2026-11-04 | **direct (historical)** |

**Not conflicts (checked against the text):** `api-contracts/README.md` line 3 ("Approval is recorded by the Integration Owner in `docs/decisions/`"): compatible if read as *recording*, ambiguous in wording; `decisions/acr-004-owner-review.md` line 62 (the Integration Owner reviews); AU-1.0 line 65 (responsible for architectural consistency); Agent Specification lines 507 and 520 and Geometry Editing Specification line 860 (module acceptance, see above). The Agent Development Specification never mentions a Project Owner, so it neither grants nor denies owner authority; it predates the owner records. A possible overlap that is **not asserted as a conflict**: "release acceptance" (Agent Specification 3.1) versus the owner's acceptance of release milestones in `RELEASE_CRITERIA.md`; to be clarified together with X-2.

Authority order stated in `api-contracts/README.md`: AU-1.0 > Geometry Editing Specification > Agent Development Specification > the contracts; the owner's AOD records are not in that list. **Owner ruling needed (folded into X-2, D-REG):** declare that for contract approval states "approve" means the Project Owner and that the Integration Owner reviews, validates, integrates, records and gives module acceptance; name the documents to amend (Agent Specification line 69, Task Pack lines 6 and 710; the audit-resolution lines by erratum); or delegate named classes of approval to the Integration Owner explicitly. Until then no agent treats an Integration Owner statement as a contract approval.

**Change-control difference (also for the owner):** `owner-architecture-decisions-v1.md` section 4 accepts the owner's "explicit approval in chat or in a signed-off pull request" to change an AOD, while plan 3.3 says a chat instruction alone is **not** a confirmation source for a registry entry. The registry rule (3.3) is the stricter one and is used here.

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

Totals: **15 decisions in GOV-01** (checked against the GOV-01 task row and checklist rows 1-15; the count is unchanged by this pass). Blocking PS-0: 11. Blocking PS-1: 2. Deferred: 2. Personal choice: 9; confirm-only: 6. X-1...X-4 are **not** additional GOV-01 decisions; see section 3.

## 2. Decisions

Common fields for every item: **Status:** Proposed; no registry entry; not confirmed. **Documents to change after approval (always):** `docs/decisions/decision-registry.md` (new entry), `docs/archive/PROJECT_STATUS.md` (5.5, 5.3), `docs/production/ps-0-exit-checklist.md` (row state); further documents are listed per item.

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
**Authority:** approval or rejection is the Project Owner's; the Integration Owner only reviews against sources (acr-004-owner-review.md line 62).
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
**Options (labels as in OAM-10; the label "R-A" is also used by OAM-04 for the renderer option `faces`, so registry entries must write `OAM-10/R-A`):**
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

**Question:** What is the contract of `RegionStamp` (what is summarised, how it is encoded, hashed, versioned and compared), and which risk policy applies to a hash collision?
**Sources:** `architecture-blocker-decisions.md` section 1 (OD-R6: "stable 128-bit hash over sorted `(CellKey, revision)` plus count; snapshots keep exact per-cell revisions"); `api-contracts/revision-and-snapshot-semantics.md` (section 1 table: `RegionStamp` = set of `(BrickKey, BrickRevision)`, derived, not stored; open item OD-R6 "set or hashed"); `processing-contracts.md` PR-02 (open: per-brick vs per-region stamp); CORE-05 and PROC-02 in `agent-task-breakdown.md`. The spike does not implement it.
**Why decide:** under R-A (item 5) asset and layer revisions advance with every streamed chunk, so conflict detection and derived-data staleness must rely on brick-level information; `RegionStamp` is the summary used for that. C-16 (P0-alpha) mentions it.
**What the contract must specify (none of the values is decided):**

| # | Requirement | Proposal in the sources / this review | Status |
|---|---|---|---|
| 1 | Stamped set | `S = {(BrickKey, RevisionToken)}` for bricks intersecting region R at snapshot time; the token includes the epoch | Proposed |
| 2 | Hash algorithm | not named in any source | **open** |
| 3 | Digest size | 128 bit (blocker text) | Proposed |
| 4 | Canonical encoding of the input | fixed-width little-endian bytes, independent of in-memory layout: `layer u64, level i8, x i32, y i32, z i32, epoch u32, value u64` per entry = 33 bytes; no padding | this review (to be accepted) |
| 5 | Sort order | ascending total order on `BrickKey` as the tuple `(layer, level, x, y, z)`, signed integer comparison | this review |
| 6 | Cardinality | the element count (u64) is hashed in the prefix and stored in the stamp | blocker text + this review |
| 7 | Domain separation | a fixed ASCII domain tag (for example `cube.regionstamp`) and the encoding version are hashed first; other hash uses use different tags | this review |
| 8 | Versioning | the stamp carries `(algorithm id, encoding version)`; stamps of different versions are **never equal** (the result is "unknown/changed", never "unchanged") | this review |
| 9 | Empty region | a defined stamp (count 0, tag, version) | this review |
| 10 | Collision consequence | a changed region could look unchanged | policy below |
| 11 | Exact verification | possible: snapshots keep exact `(BrickKey, revision)` per brick; cost O(N) for N bricks | blocker text |
| 12 | Cost | computing: O(N) over the N intersecting bricks, **not incremental** with a sorted-order hash; input = 33 bytes x N + prefix; stored stamp = digest (16 B) + count (8 B) + version fields. Throughput on target devices is **unmeasured**; a benchmark is required, no number is claimed | open |

**Collision risk policy (owner choice; the stamp must never silently produce "unchanged" for a changed region without an accepted policy):**
* **P-1 exact-on-equal:** a stamp match may not by itself authorise an authoritative or irreversible action (commit validation, publishing a derived result as current, accepting a recovery point); on a match the implementation also compares the exact per-brick revisions (O(N)). A *mismatch* is conclusive for the same algorithm and version. + no false "unchanged" in authoritative paths; - O(N) cost on the equal path.
* **P-2 accept residual probability:** the owner records the accepted risk for named uses only (for example client-side cache hints). For a random 128-bit digest the probability is about 2^-128 per comparison **if the function behaves like a random function and nobody can choose inputs adversarially**; the owner must state that assumption.
* **P-3 hybrid (this review's candidate):** P-1 for authoritative uses, P-2 for cache hints only. The owner decides which uses are "authoritative".

**Algorithm candidates (none is chosen):** (a) **BLAKE3 truncated to 128 bits**: cryptographic design, fast, supports keyed and derive-key (context) modes useful for domain separation; needs an external crate in Core, which is a D-E dependency-policy question (the spike Core has no dependencies); its licence and `cargo deny` result must be checked at adoption, not assumed here. (b) a non-cryptographic 128-bit hash (for example xxh3-128): faster, no adversarial resistance; acceptable only for P-2 uses. (c) the standard-library default hasher: **excluded**, its algorithm is unspecified and may change between Rust versions. (d) an order-independent combination of per-brick hashes (XOR or addition): incremental but algebraically weak; not suitable for authoritative uses.
**Different hash roles (one algorithm is not assumed to serve all):** (1) `RegionStamp` (this item); (2) content identity, `content_hash` of an asset view (CORE-11, a PS-1 task; algorithm **also unnamed**); (3) persistence integrity (checksums, snapshot and recovery-point hashes in PERS-01 and the manifest; decided with the format, GOV-10); (4) `OperationId` is **not a hash** (D-R8, item 9). Roles 2 and 3 are not among the 15 and are listed in section 3 as open items H-1 and H-2.
**Options for what to approve now:** **A.** name algorithm, encoding and policy in the entry (needs the owner's algorithm choice). **B.** approve the *abstract* contract (rows 1, 3-9: opaque, versioned, comparison only within the same version, policy P-1 or P-3) and register the algorithm in a **later** entry that must exist before CORE-05 starts. B keeps GOV-02 unblocked but blocks CORE-05; the ADG has no task for that later entry (finding F-9 in `PS-0-REMAINING-BLOCKERS.md`).
**Impact:** CORE-05, PROC snapshots/dependency revision (PR-02), NET, client cache, recovery check.
**Deferral consequence:** with neither A nor B, C-16 cannot be approved (P0-alpha).
**Documents to change:** `core-contracts.md` C-16, `revision-and-snapshot-semantics.md` (RegionStamp row and OD-R6), `processing-contracts.md` PR-02 note, `module-boundaries.md` (a task or a GOV-02 clause for the algorithm entry, if option B).

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

**Question:** `CellKey{layer, level: i8, coord: i32x3}` with brick edge 16 as a named constant, as the Provisional PC-1 basis?
**Sources:** blocker decisions (D-K); `agent-task-breakdown.md` CORE-02 (`CellKey`, `Region`, `VoxelPos`; tests "negative-coordinate symmetry, level round trip"); production plan 5.3 and 4.3 (PC-1 basis BL-1); `core-contracts.md` C-07 open items "int width, refinement addressing"; spike `coord.rs` (verified: `BRICK_SIZE: i32 = 16`, `i32` coordinates, `div_euclid`/`rem_euclid`, test `negative_coordinates_floor`). Voxel Architecture **absent**.
**Why decide:** every crate and every persisted byte depends on key widths and the brick edge; it is the assumption of Provisional PC-1.
**What D-K must fix (checklist for the entry):**

| Topic | State in sources | Gap |
|---|---|---|
| Spaces | `VoxelPos`, `BrickCoord`, `LocalPos`, `CellKey` (= BrickKey) | `VoxelPos` width **unspecified** (glossary 0.1) |
| Negative coordinates | Euclidean (floor) division and non-negative remainder; spike-proven | none; keep as a contract rule |
| Level (refinement) | `level: i8`, level 0 = the layer's base resolution (plan 5.3) | **direction** (does +1 mean finer or coarser?) and the **mapping between coordinates of different levels** are unspecified; C-07 lists "refinement addressing" as open |
| Range | brick coordinate `i32`: +-2^31 bricks | the derived voxel coordinate (brick x 16 + local) **does not fit `i32` when the brick coordinate exceeds about 1.3e8 (2^27)**. In the spike the multiplication is `i32 * i32` (`coord.rs`: `x * BRICK_SIZE + local`), so the effective voxel range is +-2^31 voxels, not +-2^31 bricks. The two widths must be decided together |
| Brick edge | 16 | no benchmark compares edges; the sources require only the RG-1 diff against the Voxel Architecture; **a benchmark is this review's proposal, not a sourced requirement** |
| Serialization | not defined (formats not approved) | persisted key encoding must be fixed-width, endian-fixed, schema-versioned and independent of the in-memory type (principle P7); decided with GOV-10 |
| World size | "practically endless world" (project requirement) | see below |

**World-size limitation (stated, not hidden).** A fixed-width integer coordinate gives a **bounded** world. With voxel coordinates in `i32` the extent is 2^32 voxels per axis; in metres it is `2^32 x voxel_size`, and the production voxel size (a per-layer value, `voxel_size_m`) is not in the sources, so the extent cannot be computed here. For orientation only: at an *assumed* 0.1 m voxel the extent would be about 4.3e8 m, roughly ten times the circumference of the Earth (about 4.0e7 m); this is an illustration, not a project figure. "Practically endless" is therefore defensible only if the owner accepts a **stated finite bound**, or if the model adds a coordinate system that goes beyond one fixed key (for example `i64` coordinates, or a region id plus a local offset so that numeric ranges stay small while the world grows, which also matches partitioning a world across servers). Which of these satisfies the requirement is a scope decision of the owner. Related but separate: single-precision rendering far from the origin needs a camera-relative scheme (an adapter concern, not part of D-K).
**Options:** **K-A.** keep `i32` brick coordinates and **restrict the valid range** so that `VoxelPos` fits `i32` (|brick| <= 2^27); document the bound; validate at the API boundary. **K-B.** `VoxelPos` `i64`, brick coordinate `i32` (derived voxel coordinates widen; larger arithmetic and wire size for positions, not for keys). **K-C.** both `i64`: effectively unbounded; - double key size in memory, persistence and wire; unmeasured. **K-D.** hierarchical key: `RegionId` (i32x3) + local `CellKey`: range and server partition in one model; - more complex addressing, a new concept in Core. Edge alternatives 8 or 32: no data.
**Consequences:** whichever option is chosen is recorded as **Provisional PC-1** with revalidation gate **RG-1** (diff against the Voxel Architecture when supplied) and a benchmark review; formats do not depend on PC-1 until RG-1 (GOV-10).
**Impact:** C-07, CORE-02/03, backends (PS-5), persistence format (PS-10), worldgen, network.
**Deferral consequence:** CORE-01/02/03 (keys, bricks) cannot start; the PC-1 register entry has no basis.
**Documents to change:** `core-contracts.md` C-07 (+ `VoxelPos`), `production-architecture-plan.md` 4.3 and 5.3, `api-contracts/README.md`.

### 11. D-T — Authoritative transform and numeric policy

**Question:** Is the authoritative `GeometryTransform` an integer translation + one of the 24 axis-aligned rotations + power-of-two scale, and **to which data does "no floating point" apply**?
**Sources:** blocker decisions (D-T: "no floating point in authoritative or persisted state"); `core-contracts.md` C-04 (open "f32/f64, epsilon"); spike `voxel_size_m: f32` (verified in `asset.rs`); `integration-and-qa-plan.md` section 3 (archcheck rule "no floating point in authoritative or world-defining code per D-T"); `agent-task-breakdown.md` (cube-worldgen forbidden scope "floats in world math"); plan 5.2 (closed `LayerData::{Voxel, Sdf, Mesh}`); the spike generator `crates/cube-worldgen/src/*.rs` contains **no `f32`/`f64`** (verified by grep, 0 matches).
**Why decide:** determinism across platforms is required by server-authoritative play, cross-play, replay and stable hashes; C-04 is in P0-alpha.
**The first wording of D-T is ambiguous.** "Authoritative or persisted state" can be read to cover mesh vertices, SDF samples, entity positions and gameplay physics. Seven classes must be kept apart:

| # | Class | Covered by D-T as proposed? | Rule needed |
|---|---|---|---|
| 1 | World/Asset/Layer coordinates (voxel and brick positions) | yes: integers (D-K) | integer only |
| 2 | `GeometryTransform` (C-04) | **yes, the target of D-T** | integer translation, 24 rotations, power-of-two scale; no float |
| 3 | Mesh vertex coordinates | **no**: representation unspecified in the sources (`Mesh(PageStore)`) | own numeric policy: float with canonical form, or fixed-point/integer in a layer-local grid |
| 4 | SDF/Density samples | partly: D-C proposes **quantised** density (integer); the SDF distance representation is unspecified | own numeric policy |
| 5 | Authoritative gameplay state (entity positions, physics, combat) | **unclear**: the sentence can be read to include it, the geometry documents do not define gameplay | outside D-T unless the owner extends it; needs its own policy before gameplay planning (roadmap gap R-02) |
| 6 | Persisted geometry representation | depends on 1-4 | canonical byte encoding per representation (principle P7) |
| 7 | Rendering-only derived data (GPU meshes, normals, LOD) | no | floating point allowed; never authoritative, never hashed, always re-derivable |

**Requirement for classes 3, 4, 5 (and any later representation):** a separate numeric policy before the representation's contract may be approved, stating: precision; canonicalization (one representation per value, for example -0, NaN, denormals, ordering); serialization (byte layout, endianness, version); cross-platform determinism (what is guaranteed and how it is tested; **floating-point results are not deterministic across compilers, CPUs, fused multiply-add use or library functions unless the operations are specified and tested**, so determinism is not assumed for Mesh/SDF computations); comparison and tolerance (exact equality, or an explicit tolerance and who applies it); conversion and resampling rules between representations (never implicit; rounding mode named). **No numeric parameter is proposed here**; none has a source.
**Options:** **A.** integer transform for classes 1-2 as proposed, with the scope sentence "D-T covers coordinates and transforms; classes 3-5 need their own numeric policies" (recorded as an *amendment*). **B.** A with the original broad wording (risk: unimplementable for Mesh, unclear for gameplay). **C.** `f64` with an epsilon policy: + general transforms; - platform-dependent results, unstable hashes, epsilon rules in every comparison.
**Impact:** C-04, CORE-02, archcheck rule text in XT-01/XT-02 (must name the data classes it forbids floats in), SDF/Mesh backends (PS-5), worldgen rule (the spike generator is already integer), client adapter (class 7 stays float).
**Deferral consequence:** C-04 cannot be approved (P0-alpha).
**Documents to change:** `core-contracts.md` C-04, `integration-and-qa-plan.md` section 3 (archcheck rule wording), `production-architecture-plan.md` (Core model), `agent-task-breakdown.md` (cube-worldgen forbidden scope wording).

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

**Question:** What is the single development-environment policy: toolchain, edition, CI host, project licence, dependency-licence allow-list, panic strategy, FFI panic boundary, async/runtime policy and development dependencies?
**Sources:** blocker decisions (D-E); `integration-and-qa-plan.md` section 3 (required checks); AOD-23 (licence policy for *references*, not for dependencies or for Cube itself); `agent-task-breakdown.md` ADP-03 ("Error wrapping, catch_unwind, panic policy test (including panic=abort decision)", PS-12); production plan risk table (line ~276: "`panic = "abort"` across GDExtension | crashes | ADP-03 test before adoption"); root `Cargo.toml` of the spike (verified: edition 2021, `[profile.release] panic = "abort"`, `license = "UNLICENSED"`, `publish = false`); godot-core 0.5.5 registry source (verified: `src/init/mod.rs` calls `std::panic::catch_unwind`); local `cargo 1.97.1` (an observation, not a decision).
**On the supposed contradiction.** D-E itself is consistent: it proposes `catch_unwind` at the FFI **and** says to test `panic = "abort"` *before adopting it* (abort is not adopted by default). The contradiction is between the proposal and the **spike's** `[profile.release] panic = "abort"`, under which `catch_unwind` cannot catch anything. The spike profile is not production policy; production has no `Cargo.toml` yet.
**Parameters that need the owner's decision (no value is chosen here):**

| # | Parameter | Source state | Options and consequences |
|---|---|---|---|
| a | Rust toolchain version | none; "pinned stable" only | pin an exact stable version in `rust-toolchain.toml` (reproducible; upgrades are deliberate PRs) / track latest stable (no reproducibility). The minimum supported Rust version of the dependencies (godot crate 0.5.x) must be checked when pinning |
| b | Rust edition | spike uses 2021 | 2021 (matches spike code) / a newer edition (language changes; no sourced reason; needs a check that all dependencies and `godot` support it) |
| c | CI host/platform | not stated; "required status checks" implies a hosted pipeline | GitHub Actions hosted runners / self-hosted runners / another service. Consequences: available OS images (Windows is the development OS), cost, secrets, later Android/iOS runs, how branch protection is configured |
| d | Project (Cube) licence | spike manifest says `UNLICENSED` (verified) | proprietary (all rights reserved) / an open-source licence / undecided with an explicit note. Determines which dependency licences are acceptable |
| e | Dependency-licence allow-list | "owner's decision"; AOD-23 covers only references (Veloren GPL etc.) | categories to decide: permissive; weak copyleft (file-level or linking obligations); strong copyleft; plus exceptions per crate. Depends on (d) and on mobile store distribution. Enforced by `cargo deny` (GOV-06). **No list is proposed in this register** |
| f | Panic strategy per artifact | spike: abort | `unwind` for the GDExtension `cdylib` (so FFI `catch_unwind` works) and `abort` or `unwind` for server/test builds / `abort` everywhere (no catch at the FFI; a panic ends the process, acceptable only if the engine process may die). Cargo sets the strategy per final artifact profile, so the extension and the server may differ |
| g | FFI panic boundary | `catch_unwind` at every entry point (D-E) | catch and convert to error codes (needs unwind) / forbid panics by construction (`abort`) and treat any panic as fatal; tested in ADP-03 (PS-12) either way |
| h | Async/runtime policy | no async runtime in Core, Processing, Backends; std-thread `Executor` (D-E) | as proposed / allow a runtime in lower layers (couples the pure core to a runtime, harder loom tests). **Open:** the runtime of the server and network layers (GOV-12/GOV-22; not covered by D-E) |
| i | Development dependencies | `proptest`, `loom`, `trybuild`, `criterion` (D-E) | accept / replace individually; each must pass `cargo deny` under (e); `loom` and `trybuild` are needed by the commit-gate and mutation-boundary acceptance tests |
| j | Dependencies in Core | none stated for production; the spike Core has none (verified) | allow pure, audited external crates in Core (for example a hash crate for OD-R6) / forbid all (the hash is then implemented in-repo). Needed for item 8 |

**Why decide:** GOV-05 (CI specification) must name tools, versions and checks; GOV-06 implements them; XT-01 relies on the dependency policy.
**Impact:** GOV-05, GOV-06, XT-01, every crate, ADP-03.
**Deferral consequence:** the CI specification cannot be approved; G0 stays open. Parameters f and g may be deferred in writing to ADP-03 (PS-12) **only if** CI and the production profile do not set `panic` before then.
**Documents to change:** `integration-and-qa-plan.md` section 3, `rust-toolchain.toml` / `deny.toml` / workflow files (created later by GOV-06; not now), `module-boundaries.md` (execution-model notes).

### 15. Branch option for the docs-only baseline

**Question:** How does `master` receive a docs-only baseline?
**Sources:** `integration-and-qa-plan.md` section 1; GOV-04 (acceptance: "no file of the spike code list is on master"); roadmap-v1 section 7 (spike code not for `master`).
**Facts (verified):** `master` has 13 files; the docs branch is 35 commits ahead and changes 268 files; the docs branch contains the spike crates, `godot-client/`, `scripts/`, `docs/spike/` (170 files). Production crate names (`crates/cube-core`, ...) are the same as the spike crate names, so mixing them would collide.
**Options:** **A.** a new docs-only branch from `master` with the docs commits (links into `docs/spike/` removed, or spike reports added by a separate docs commit). + clean base; spike stays on its own branches. − the docs commits must be re-created (cherry-pick of path-limited changes), so history differs from this branch. **B.** path-limited PRs. − GitHub PRs carry the whole branch diff; to limit paths you create separate branches that contain only those paths, which is option A by another mechanism. **C.** accept spike code as an explicit experimental directory. + no re-creation. − contradicts roadmap-v1 section 7 and the "spike never merges" rule; collides with production crate paths.
**Impact:** GOV-04, GOV-06, every production PR base.
**Deferral consequence:** no production base; GOV-04/06 blocked.
**Documents to change:** `integration-and-qa-plan.md` section 1 (state), `PROJECT_STATUS.md` 5.2.

---

## 3. X-1...X-4: where they live in the governance process (CR-02)

**Check of the structure (verified in `agent-task-breakdown.md` and `module-boundaries.md` Part C):** the GOV-01 task row lists exactly 15 decisions (ACR-001/002/003, ACR-004 part A, OAM-10 principle, OD-R1, OD-R2, OD-R6, D-R8, D-K, D-T, D-C, D-0, D-E, the branch option) and `ps-0-exit-checklist.md` section 2 rows 1-15 match it item for item. The first version of this register called X-1...X-4 "adjacent decisions"; that wording is corrected: only X-1 has an unresolved place in GOV-01, and X-2...X-4 are outputs of other G0 tasks that already exist.

| Item | What | Where it lives in the ADG | In GOV-01's 15? | Own owner confirmation? | Written deferral allowed? | Blocks |
|---|---|---|---|---|---|---|
| X-1 | Confirmation of the recorded decisions AOD-01...24 and OAM-01/02/03/07/08 (agent-transcribed from chat) | **gap**: plan 3.3 says "decision GOV-01 covers this"; the GOV-01 task row and checklist do not list it | no (not one of the 15); it is a **confirmation batch** in GOV-01's acceptance, not a 16th decision | yes: one registry entry per record, countersigned by the owner (signed commit or owner-written file) | the default state is "recorded, awaiting confirmation"; a written deferral per record is possible, **but** the records that justify other items must be confirmed first: AOD-20 (items 4, 5), AOD-18 and AOD-24 (item 13) | GOV-02 (P0-alpha approval rests on AOD-18/20/24) |
| X-2 | D-REG: adopt the status model (Proposed, Provisional, Owner-Approved, Implemented, Frozen, Rejected, Superseded), the registry, **and the authority split of 0.2** | **GOV-02** row ("adopt the Contract Status Model"); blocker doc section 4 maps D-REG to "GOV-01, GOV-02" | no | yes (registry entry) | no: GOV-02 cannot approve P0-alpha without a status model | GOV-02 |
| X-3 | Owners for WorldGen, Networking Adapter, Server Runtime, Client Runtime, Engine Adapter, Import/Export, O-1, O-2 | **GOV-03** (independent G0 task; S and I empty) | no | yes | **yes, explicit in the task row: "or defer each in writing"** | no G0 task; the tasks of the unassigned modules (PS-11+, SPK-NET) |
| X-4 | Owner approval of the CI specification | **GOV-05** (G0 task; S and I = GOV-01) | no | yes ("Owner-Approved CI specification") | no (G0 member) | GOV-06, TEST-01, XT-01 |

Decision: X-1 is the only item that needs an explicit home. Proposal (owner to accept): record it as part of GOV-01's acceptance criteria. X-2 stays in GOV-02, X-3 in GOV-03, X-4 in GOV-05. The number of GOV-01 decisions stays 15; no new task is created.

### 3.1 Which governance task needs which decision (technical need versus ADG gating)

| Task | Needs (technically) | ADG gating (verified) |
|---|---|---|
| GOV-02 | items 1, 4, 5, 6, 7, 8 (or its abstract form), 9, 11, 13; items 10 and 12 as the stated basis of PC-1; X-1 (AOD-18/20/24), X-2 | merge-after GOV-01 (all of it) |
| GOV-03 | none of the 15 | no dependency |
| GOV-04 | item 15 | merge-after GOV-01 |
| GOV-05 | item 14 (parameters a, b, c, e, i, j; f, g may be deferred to ADP-03), item 15 (CI runs on the chosen base) | start-after and merge-after GOV-01 |
| GOV-06 | outputs of GOV-04 and GOV-05 (hence items 14, 15); owner permission to push, open a PR and enable branch protection | merge-after GOV-04, GOV-05 |

Because the ADG gates GOV-02/04/05 on **all** of GOV-01, a partial answer unblocks nothing formally. Without changing the ADG the owner may answer in two batches inside the one GOV-01 task (batch 1: items 14, 15, which let GOV-04/05 be prepared; batch 2: the Core items), and mark GOV-01 Accepted only when both are recorded.

### 3.2 Open items found by this pass (not new GOV-01 decisions; each needs an owner or a task home)

| ID | Item | Why | Proposed home |
|---|---|---|---|
| H-1 | hash algorithm and encoding of `content_hash` (CORE-11, a **PS-1** task; golden hashes depend on it) | CORE-11 cannot be implemented without it | the OD-R6 entry (option B) or a sibling entry; before CORE-11 starts |
| H-2 | persistence checksum / snapshot-hash algorithm | needed by PERS-01 and the AOD-21 verify step | GOV-10 (format approval), already in the ADG |
| H-3 | Provisional status for PR-01...PR-08 and E-02...E-12 (CR-10): PS-2 and PS-3 production tasks implement contracts that are only **Proposed** until GOV-07/GOV-09 (which run after the implementation); plan 3.1 says a Proposed contract may be used for "design, review, mocks, spikes" and that "no production module may claim it" | see `GOV-01-CORRECTIVE-REVIEW.md` CR-10 | an owner entry registering a Provisional group (named PC-2 and PC-3 in the corrective review) at the start of PS-2 and PS-3, or a change of the ADG so that production merges wait for GOV-07/GOV-09 |
