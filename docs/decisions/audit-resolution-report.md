# Cube — Audit Resolution Report (Geometry Architecture)

**Prepared by:** Main Agent / Integration Owner role. **Basis:** Architecture Update v1.0 (AU-1.0).
**Status:** report for owner review. Contains Architecture Change Requests (ACR) and a list of decisions that need the project owner.

Sections follow the requested structure A–J. Sections B (ownership), C (dependency matrix), D (contracts), E (revision semantics) and F/G (task pack, QA pack) are stored in their working documents and summarised here.

---

# A. Audit Resolution Report

Format per item: Original Finding · Approved Resolution · Updated Document · Implementation Implications · Remaining Issues.

| # | Original finding | Approved resolution (AU-1.0) | Updated document | Implementation implications | Remaining issues |
|---|---|---|---|---|---|
| 1 | No Geometry Core owner | **Core Agent** (§2) | Spec §2.2/§23; Task Pack §2.5; QA §4 | Core is stage 1 and a full implementation task, not an integration of something existing | none |
| 2 | No Processing owner | **Processing Agent** | same | Processing is L1, prerequisite of Orchestration, backends, Tools, Persistence | Who owns concrete derived-data providers (Surface Nets, greedy meshing, collision, hitbox, LOD) is not assigned — **O-1** |
| 3 | No Composite owner | **Composite Agent** | same | operations as commands through `IEditingService` | Data-model placement: **ACR-003** |
| 4 | No Resampling/Resolution owner | **Resolution Agent** | same | Resampling and Refinement are L5 tools | transition algorithm Open |
| 5 | Processing in the wrong place in the pipeline (stage 8) | Processing is a base layer (§3.1 layer 2) | QA §5 (Processing = stage 2), Spec §22 | all async-dependent modules wait for Processing contracts only, not the whole implementation (contracts first) | none |
| 6 | History ↔ Persistence cycle | Independent abstractions in a lower layer (§3.2) | QA §1.2 notes 2–3; `editing-contracts.md` §0 | `IDeltaStore`, `ISnapshotStore`, `ITransactionObserver`, `GeometryDelta`, `Checkpoint` in L2; neither module imports the other | durability ordering of observer vs ack (E-12), who owns the operation journal (**O-3**) |
| 7 | Orchestration dependencies insufficient / inconsistent | "Clarify in contracts" (§22) | QA §1.2 note 1, §1.3 rows 4–7; Task Pack Agent A | Orchestration: Core, Processing, API direct; Backends and History as abstractions only | nested / async transactions Open (E-04) |
| 8 | Task Pack assumed existing Core | Fix the order (§22) | Task Pack §13 and §2.5; Agent H wording | no agent may assume Core; Phase 2 is a sequential gate | Core brick export view must be defined by Core Agent (C-08) |
| 9 | Missing contracts: Transform, Bounds, Region, ModifierStack, Junction, RefinementRegion, BrickStorage, Delta, Checkpoint, AuthorizationContext, OperationId | "Add" (§22) | `api-contracts/` C-04, C-05, C-06, E-08, C-13, C-12, C-08, E-09, E-10, E-02, C-17 | all drafted at specified level; each has named Open decisions | not approved; ModifierStack evaluation has no owner (**O-2**) |
| 10 | RevisionToken semantics undefined | Develop and agree (§22, §7.2) | `revision-and-snapshot-semantics.md` | proposal covers levels, increments, snapshot cut, atomic commit, conflicts, stale rejection, History/Persistence/Networking | **proposal only**; OD-R1…R8 |
| 11 | No network architecture | Networking boundary (§20) | `editing-contracts.md` E-13; QA §1 L9, §5 stage 11 | boundary and server-authority semantics specified; no protocol | Networking Adapter has **no owner (O-3)**; protocol/transport Open |

Additional contradictions found while aligning the documents (not in the first audit):

| # | Finding | Resolution |
|---|---|---|
| 12 | QA Pack §2 gave `RevisionToken` and `DirtyRegion` to "Core / Processing" (two owners) | Single owner Core (ACR-001) |
| 13 | AU-1.0 §23 lists `Dirty Region`, and implicitly `OperationId`/`RevisionToken`, under Processing/Editing, but Core must track dirty inside Bricks and Core may not depend on them | Foundation types in Core (ACR-001) |
| 14 | AU-1.0 §23 lists Composite Part and Assembly Graph under Geometry Core while §2 gives Composite its own agent | Data model in Core, operations in Composite (ACR-003, pending confirmation) |
| 15 | QA Pack stage 1 asked Core for "serialization primitives", Task Pack H said "reuse Core serialization interfaces", AU-1.0 §23 lists Brick Serialization under Persistence | Core exports/imports views; Persistence owns format (updated in QA, Task Pack H, `persistence-contracts.md`) |
| 16 | QA matrix listed QA as Required dependency of all modules, contradicting "QA is not a runtime dependency" | Clarified as test-time only |
| 17 | Spec §16 module list omitted Resampling and Composite | Updated |
| 18 | Task Pack required History for Sculpting/Boolean (old matrix) | Removed; History subscribes via `ITransactionObserver` |
| 19 | AU-1.0 §2 gives no owner to Networking Adapter, Engine Adapters, Import/Export adapters, Surface/Collision/LOD providers, Modifier evaluation | Opened as O-1…O-5 |
| 20 | Voxel Architecture v1.0, GDD/TDD v0.3 and Geometry Processing Specification are not in the repository | Cannot be updated here — see section H item O-6 and section J |

---

## A.1 Compile-time acyclicity vs runtime back-paths

- **Compile-time graph: no cycles.** Verified by script over the grid in QA Pack §1.2 (13 modules; abstraction edges resolved to the L2 layer). This is a one-time check; no automated test exists yet.
- **Runtime back-paths exist, by design, through ports:** `Orchestration → ITransactionObserver → History → IEditingService → Orchestration`; `Tools/Composite/Networking → IEditingService → Orchestration → IGeometryBackend → Backends`; `History → IDeltaStore/ISnapshotStore → Persistence` (one-way). These are legal because every arrow points to an L2 contract, not to an implementation.
- Therefore "no cycles" is a statement about imports only. The runtime call graph contains loops that rely on correct port design (reentrancy, ordering of `ITransactionObserver` delivery). Those must be covered by integration tests (QA Pack §1.5).

# B. Updated Module Ownership

Unchanged from AU-1.0 §2 (one owner each): Core Agent, Processing Agent, Main Agent/Integration Owner (Editing API & Contracts), A (Orchestration), B (Voxel), C (SDF), D (Mesh), E (Sculpting), F (Boolean), G (History), H (Persistence), I (QA), Composite Agent, Resolution Agent. **Unassigned:** Networking Adapter, Engine Adapters, Import/Export adapters, derived-data providers, modifier evaluation. Full table: QA Pack §4.

# C. Dependency Matrix

QA Pack §1 (grid, provider/consumer/interface/direction/ownership/lifetime/revision/threading table, acyclicity argument).

# D. API Contract Matrix

`docs/api-contracts/README.md` §3. 45 contracts + 3 interface boundaries (PR-09, E-13, P-09) + 2 unnumbered storage ports; all Draft; 5 tool-level contracts (BooleanPolicy, Sculpt/Brush, Resample/Refine ops, Composite ops, History interface) are outside this pass and marked Missing.

# E. Revision & Snapshot Semantics

`docs/api-contracts/revision-and-snapshot-semantics.md`. Key proposed rules: monotonic scoped counters with Epoch; exactly +1 per scope per commit; Region revision derived from brick stamps; commit validation at brick granularity under a short Asset commit gate; snapshots are consistent cuts of COW pins; publish validation is atomic with respect to commits; undo is a forward commit; recovery bumps Epoch; client predictions live in a separate epoch.

# F. Updated Agent Task Pack

Task Pack rev 1.1: new Core, Processing, Composite, Resolution sections; per-agent dependency and provides lines; new dependency table; execution order with Phase 0 readiness; Agent G and H scope corrections.

# G. Updated Integration & QA Pack

QA Pack rev 1.1: dependency grid, ownership matrix, 14-stage pipeline with exit conditions and allowed parallelism, new CI checks (dependency-direction, engine-independence, contract-version), updated test matrix and limitations.

---

# Architecture Change Requests

These do not change an approved decision; they place contracts where AU-1.0's own dependency rules require. Each needs the owner's explicit approval. Until approved, documents follow the proposed placement and mark it.

## ACR-001 — Foundation types live in Geometry Core

**Status: PENDING** (not approved, not rejected).

1. **Current approved design:** AU-1.0 §23 lists Revision Token and Operation ID under Editing, and Dirty Region under Processing. AU-1.0 §3 forbids Core depending on Processing, §5.3 requires Core to be independent, §10 requires Dirty Regions tracked inside Bricks.
2. **Problem:** Core must produce revisions and dirty regions and carry operation ids in errors; Processing validates against revisions. If the types sit in Editing or Processing, Core would depend upward.
3. **Proposed change:** `RevisionToken`, `OperationId`, `DirtyRegion`, and the Result/Error envelope are defined and owned by Core (contracts C-16…C-19). Editing API owns only the public error **category set** and result variants.
4. **Alternatives:** (a) a separate "Foundation" module below Core, owner Core Agent or Integration Owner; (b) duplicate value types in each layer (rejected: divergence).
5. **Affected modules:** Core, Processing, Editing API. **API impact:** location only. **Data compatibility:** none. **Performance:** none. **Tests:** contract tests live with Core. **Migration:** none (nothing implemented).

## ACR-002 — Orchestration reaches backends only through L2 ports (clarification)

**Status: PENDING** (not approved, not rejected).

1. **Current:** QA v1.0 matrix said Orchestration requires Backends; AU-1.0 §22 says "clarify in contracts".
2. **Problem:** direct dependency creates a heavy cycle risk with Tools/History.
3. **Proposed:** `IGeometryBackend` and capabilities defined in L2; Orchestration holds a registry. (Implemented in the QA grid.)
4. **Alternatives:** direct dependency (rejected: composition and testing cost).
5. **Impact:** Orchestration and backends test independently. Not a departure from AU-1.0; recorded because AU-1.0 says "to be clarified".

## ACR-003 — Placement of Composite data model

**Status: PENDING** (not approved, not rejected).

1. **Current:** AU-1.0 §23 lists "Composite Part" and "Assembly Graph" under Geometry Core contracts; §2 assigns Composite Geometry to a separate Composite Agent; §3 says Core cannot depend on other modules.
2. **Problem:** unclear who defines the data model versus operations and constraint logic.
3. **Proposed:** Core owns the passive data model (parts, references, graph, attachment points). Composite Agent owns operations, constraint evaluation and transform propagation.
4. **Alternatives:** (a) whole model in Composite, with Persistence depending on Composite (adds a dependency, conflicts with "Composite persists structure through manifest"); (b) Core owns everything (violates Composite ownership).
5. **Impact:** Persistence can store composite structure without depending on Composite. Needs owner confirmation.

---

# H. Remaining Open Decisions

## H.1 Requiring the project owner

| ID | Decision | Why it blocks |
|---|---|---|
| O-0a | Programming language | all contracts' ownership/lifetime/thread-safety rules; CI; test tooling |
| O-0b | Game engine | engine adapters, rendering, physics |
| O-1 | Owner of derived-data providers: Surface Nets (SDF), greedy meshing (voxel), collision, hitbox, LOD | Processing can host the graph, but someone must write providers |
| O-2 | Owner of modifier evaluation / Bake (non-destructive path) | no agent covers it |
| O-3 | Owner of Networking Adapter, operation journal, and the protocol decision | multiplayer cannot be validated |
| O-4 | Owner of Import/Export adapters | AU-1.0 §17 |
| O-5 | Owner of Engine Integration Adapters | blocked also by O-0b |
| O-6 | Add Voxel Architecture v1.0, GDD/TDD v0.3 and the Geometry Processing Specification to the repository. (Architecture Update v1.0 is now saved as `docs/decisions/architecture-update-v1.0.md`.) | cannot update or cross-check them otherwise |
| ACR-001/002/003 | approve or reject | placement of contracts |

## H.2 Technical decisions requiring evidence (AU-1.0 §24)

Integer/float types; RevisionToken details (OD-R1…R8); atomic commit algorithm; API signatures; snapshot/COW implementation; conflict-resolution policy; network protocol; serialization format; compression; performance budgets; SDF precision; Boolean algorithms; refinement transition; delta/snapshot encoding; brick index order; capability vocabulary of `IGeometryBackend`; nested/async transaction model; permission model for `AuthorizationContext`; global material library interface (no document defines it).

Each contract file lists its own Open items.

---

# I. Architecture Readiness Assessment

| Area | State | Comment |
|---|---|---|
| Ownership | Ready except 5 unassigned modules | O-1…O-5 |
| Dependency structure | **Ready** (acyclic by construction, with CI check proposed) | pending ACR-001…003 |
| Contract inventory | **Ready** | 45 contracts + 3 interface boundaries + 2 storage ports; 5 tool-level contracts missing |
| Contract specification | **Draft** | nothing approved; P0 (19) can be reviewed first, P1 (17) in parallel with Stage 1 |
| Revision / Snapshot semantics | **Proposed** | needs review and OD-R1…R8 |
| Language / engine | **Not ready** | blocks signatures, CI, test tooling |
| Core and Processing implementation start | **Conditional** | Core starts after: language decision (O-0a), P0 approved, ACR-001 decided; Processing additionally needs P1 approved |
| Remaining modules | Blocked by Core/Processing/L2 | as designed |
| Source documents | **Incomplete** | O-6 |
| Repository | no code; documentation committed on `master` (commits `init` and the rev 1.1 documentation commit) | housekeeping |

**Conclusion:** the architecture is internally consistent and ready for contract approval. Implementation can start only for Core and Processing, and only after the language decision and approval of the P0 contracts (Core) and P1 contracts (Processing). Starting the other modules before then would reproduce the problems the audit removed.

---

# J. Recommended Next Steps

1. Owner decisions: O-0a (language), ACR-001/002/003, owners for O-1…O-5 (or explicit deferral).
2. Add the missing source documents to the repository (O-6); I will then update Voxel Architecture and GDD/TDD sections (not possible today).
3. Review and approve P0 contracts (C-01…C-19), then P1 (PR-01…PR-08, E-02…E-07, E-09, E-11, E-12). Record approvals in `docs/decisions/`.
4. Resolve OD-R1, OD-R2 (width, commit algorithm) at least at proposal level — they constrain Core.
5. Set up CI skeleton including the dependency-direction check; start Agent I infrastructure.
6. Start Core Agent, then Processing Agent against approved contracts.
7. Next contract pass: BooleanPolicy, Sculpt/Brush, Resample/Refine, Composite ops, History interface — written by their owners after step 3, reviewed by the Integration Owner.
8. Commit the documentation set to `master` (baseline commit `init` already contains the four original documents) after owner review.

---

# K. Blocker Register (code remains blocked)

| ID | Blocker | State | Closes when |
|---|---|---|---|
| B-1 | GDD/TDD v0.3, Voxel Architecture v1.0, Geometry Processing Specification absent from repository | **Open** | files added; Voxel Architecture and GDD/TDD sections then updated and cross-checked |
| B-2 | Programming language not chosen (O-0a) | **Open** | decision recorded in `docs/decisions/` |
| B-3 | ACR-001/002/003 undecided | **Open** | owner approves or rejects each |
| B-4 | P0 contracts unapproved | **Open** | Integration Owner approval after B-2, B-3 |
| B-5 | No owner for Networking, Engine Adapters, Import/Export, derived-data providers, modifier evaluation (O-1…O-5) | **Open** (not blocking Core/Processing) | owners assigned or deferred explicitly |
| B-6 | RevisionToken width and atomic commit algorithm (OD-R1, OD-R2) | **Open** (proposal exists) | decision recorded |
| B-7 | Tool-level contracts (BooleanPolicy, Sculpt/Brush, Resample/Refine, Composite ops, History interface) not written | **Open** (not blocking Core/Processing) | written by owners after B-4 |
| B-8 | Core/Processing/Composite/Resolution/Editing API ownership gaps | **Closed (documented)** | owners assigned per AU-1.0 §2; reflected in all four documents |
| B-9 | History ↔ Persistence cycle | **Open — rule documented, verifying test not implemented** (L2 port contracts E-12, `IDeltaStore`, `ISnapshotStore` also Draft) | closes only when (a) dependency-direction rule DEP-01 is recorded (done, QA Pack §1.5) **and** (b) an automated test enforcing it exists and passes (not done; needs CI/tooling, outside this docs-only revision) |
| B-10 | Processing placed late in pipeline; Task Pack assumed existing Core | **Closed (documented)** | QA §5 and Task Pack §13 reordered |
| B-11 | Missing contract inventory (Transform … OperationId) | **Documented, pending approval** | 45 contracts specified + 3 interface boundaries described; approval is B-4 |
| B-12 | RevisionToken undefined | **Documented as proposal, pending approval** | formal proposal written; approval is B-6 |
| B-13 | AU-1.0 not in repository | **Partially closed**: a reconstructed copy is stored with status Proposed/Draft; the original approved file is unavailable and the copy is not byte-verified | owner supplies original or confirms the copy |

# L. P0 / P1 Readiness

Tier definitions are the single set in `docs/api-contracts/README.md` §3 (also used by the QA Pack stage gates and the Task Pack): **P0** gates Stage 1 (Core); **P1** gates Stages 2–5 (Processing, Editing API, Orchestration, Backends); **P2** gates the affected module in Stages 6–10; **P3** covers interface boundaries and unwritten tool contracts.

| Tier | Contracts | Specified | Approved | Ready to implement against? |
|---|---|---|---|---|
| P0 | C-01…C-19 (19) | Yes, Draft | **No** | **No** |
| P1 | PR-01…PR-08, E-02…E-07, E-09, E-11, E-12 (17) | Yes, Draft | **No** | **No** |
| P2 | E-08, E-10, `IDeltaStore`, `ISnapshotStore`, P-01…P-07 (9 + 2 ports) | Yes, Draft | No | No (not needed yet) |
| P3 | PR-09, E-13, P-09 (boundaries); BooleanPolicy, Sculpt/Brush, Resample/Refine, Composite ops, History interface | boundaries described; tool-level **not written** | No | No |

**Why P0 is not ready:** (1) none is approved; (2) language is undecided, so ownership/lifetime/thread-safety rules (ownership of pins, COW handles, value-type guarantees) cannot be frozen; (3) ACR-001 changes where four P0 types live (C-16…C-19); (4) RevisionToken width and commit algorithm (OD-R1/R2) constrain `BrickStorage` and the commit primitive; (5) integer/float widths for `BrickKey`, `GeometryTransform` and `GeometryBounds` are open; (6) the missing source documents may constrain C-07…C-12 (Voxel Architecture) in ways that cannot be checked.

**Why P1 is not ready:** (1) none is approved; (2) it depends on P0 for every shared type (tokens, regions, dirty regions, errors); (3) PR-07 and PR-08 depend on the unassigned provider owner (O-1); (4) E-04, E-07 and E-11 carry open decisions (nested/async transactions, capability vocabulary, signatures); (5) the Geometry Processing Specification is missing (B-1) and may constrain PR-01…PR-08.

**Readiness consequence:** Stage 1 may start only after P0 approval, the language decision and the ACR-001 decision; Stage 2 additionally needs P1 approved.

# M. Next Tasks (ordered)

1. Owner: add GDD/TDD, Voxel Architecture, Geometry Processing Specification (B-1).
2. Owner: choose language (B-2); decide ACR-001/002/003 (B-3).
3. Integration Owner: review P0 contracts against the added sources; fix conflicts; record decisions for OD-R1/OD-R2 and integer/float types (B-6).
4. Integration Owner: approve P0, then P1 contracts; set status to Approved and version to 1.0.0 (B-4).
5. Integration Owner: set up repository skeleton, CI and the dependency-direction check (grid in QA Pack §1.2) — no module code.
6. Owner: assign Networking, Engine, Import/Export, provider and modifier-evaluation owners, or defer explicitly (B-5).
7. Start Core Agent, then Processing Agent; Agent I starts test infrastructure.
8. Module owners write tool-level contracts (B-7).
9. Commit the documentation set to `master` after owner review (the default branch is `master`; documents have been aligned).
