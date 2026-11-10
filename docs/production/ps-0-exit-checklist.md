# PS-0 Exit Checklist (proposal)

**Status:** **Proposed**; a checklist, not a decision. PS-0 (Governance and Architecture Freeze) closes when gate **G0** is satisfied (ADG part C and D in `module-boundaries.md`). **State on 2026-11-09 is in the State column below; PS-0 is not complete.** Marking an item DONE needs the evidence named in the row; an agent may not mark an owner-decision row DONE without the owner's registry entry (`production-architecture-plan.md` section 3.3).
**Related:** PS-1 entry criteria (section 3), decisions list (`architecture-blocker-decisions.md` section 1), verdict (`production-plan-revision-v2.1-report.md` section 2).

## 1. Gate G0 tasks

| # | Gate task | Owner | What must exist (evidence) | Verification | State |
|---|---|---|---|---|---|
| 1 | GOV-01 | Project owner | decision-registry.md entries for ACR-001, ACR-002, ACR-003, ACR-004 part A, part D principle (OAM-10), OD-R1, OD-R2, OD-R6, D-R8, D-K, D-T, D-C, D-0, D-E, the branch option; each with a confirmation source (owner-signed commit or PR approval) | Integration Owner checks every entry has a source; no entry made by an agent | DONE: 15 decisions registered DEC-055..DEC-070 (engineering decisions under delegation DEC-001; licence DEFERRED DEC-070) |
| 2 | GOV-02 | Project owner | registry entries: P0-alpha Owner-Approved (C-01, C-02, C-03, C-04, C-06, C-16, C-17, C-18, C-19 incl. the ACR-004 part A amendments); PC-1 Provisional with the five mandatory fields; status model adopted; AOD-01…24 countersigned | registry diff reviewed; contracts README status column updated by the Integration Owner | DONE (2026-11-09): P0-alpha and PC-1 DEC-071, status model DEC-002, AOD-01..24 confirmed by the owner in chat (DEC-004..027, conditions DEC-075); registry lint: 75 entries, no id reuse, no CONFIRMED without evidence |
| 3 | GOV-03 | Project owner | ownership table naming Core Agent, Integration Owner, Agent I and, for PS-2…PS-10, the named agents; UNASSIGNED modules listed with the stage that needs each; networking and WorldGen owners (or the spike runner) named before SPK-NET / WGEN-01 start | ownership table in module-boundaries.md matches the registry | DONE (DEC-072; written deferrals in module-boundaries A.2) |
| 4 | GOV-04 | Main Agent / Integration Owner | docs-only baseline on master (no file of the spike code on master) according to the option chosen in GOV-01 | `git ls-tree` of master lists only documentation | DONE: baseline branch `dev`, commit b9df5c4, no spike code (`git ls-tree`), master unchanged (DEC-003, DEC-069) |
| 5 | GOV-05 | Main Agent / Integration Owner | CI specification Owner-Approved: checks, required statuses, tool pins, licence allow-list | Owner-Approved document | DONE: `docs/production/ci-specification.md` (DEC-073) |
| 6 | GOV-06 | Agent I (QA and Benchmarks) | running CI: workflow-run URL green on an empty skeleton and red on seeded failures; branch protection on master lists the required checks (API output) | run URLs and protection settings attached | DONE (2026-11-09): CI green on the skeleton (run 42609049921), on tooling (42609627890) and on dev (42609915509), red on seeded failures (42609650840); ruleset `protect-dev-master` (id 26848724) active on `dev` and `master`: deletion blocked, force push blocked, required checks fmt, clippy, test (ubuntu-latest), test (windows-latest), deny, archcheck (verified via `GET /rules/branches/{dev,master}`); bypass list not readable anonymously (the owner configured it empty) |
| 7 | TEST-01 | Agent I (QA and Benchmarks) | `cube-testkit` skeleton merged and used by at least two crates | CI run | DONE: `cube-testkit` used by `cube-integration-tests` and `xtask` (run 42609627890 green) |
| 8 | XT-01 | Main Agent / Integration Owner | `xtask archcheck` v0 merged; seeded violations fail; registered as a required status check | CI run with seeded failures | DONE (2026-11-09): `xtask archcheck` v0 with seeded-violation self-tests merged (11 tests; red run 42609650840) and a required status check on `dev` and `master` |

## 2. Decisions that GOV-01 must contain (each needs an owner-confirmed registry entry)

| # | Decision | Document | Options in short | State |
|---|---|---|---|---|
| 1 | ACR-001 foundation types in Core | blocker-decisions section 1 | approve / `cube-foundation` crate | DECIDED DEC-055 (delegated) |
| 2 | ACR-002 ports in L2 | blocker-decisions section 1 | approve / direct dependency (rejected earlier) | DECIDED DEC-056 (delegated) |
| 3 | ACR-003 composite data in Core | blocker-decisions section 1 | approve / all in Composite / all in Core | DECIDED DEC-057 (delegated) |
| 4 | ACR-004 part A (Core prerequisites) | blocker-decisions section 2 | approve / amend / reject | DECIDED DEC-058 (delegated) |
| 5 | Revision model principle (ACR-004 part D, OAM-10) | blocker-decisions section 2 | R-A (recommended) / R-B / R-C | DECIDED DEC-059 (delegated) |
| 6 | OD-R1 token width | blocker-decisions section 1 | `u64`/`u32` / other | DECIDED DEC-060 (delegated) |
| 7 | OD-R2 commit algorithm | blocker-decisions section 1 | commit gate over immutable state / cell CAS | DECIDED DEC-061 (delegated) |
| 8 | OD-R6 region stamp | blocker-decisions section 1 | 128-bit hash / set / coarse | DECIDED DEC-062 (delegated) |
| 9 | D-R8 OperationId format | blocker-decisions section 1 | composite id / random / client-supplied | DECIDED DEC-063 (delegated) |
| 10 | D-K key, widths, brick edge | blocker-decisions section 1 | level `i8`, `i32`, edge 16 / alternatives | DECIDED DEC-064 (delegated) |
| 11 | D-T transforms | blocker-decisions section 1 | integer transform / f64 | DECIDED DEC-065 (delegated) |
| 12 | D-C channel model | blocker-decisions section 1 | built-in channels, quantised density / alternatives | DECIDED DEC-066 (delegated) |
| 13 | D-0 P0 scope | blocker-decisions section 1 | P0-alpha + PC-1 / wait / full P0 on baseline (contradicts AOD-24) | DECIDED DEC-067 (delegated) |
| 14 | D-E toolchain, execution model, licence allow-list | blocker-decisions section 1 | pinned stable, no async runtime in lower layers / alternatives | DECIDED DEC-068 (delegated) |
| 15 | Branch option for the docs-only baseline | integration-and-qa-plan section 1 | new docs-only branch / path-limited PRs / accept spike code (contradicts roadmap-v1) | DECIDED DEC-069 (delegated) |

## 3. PS-1 entry criteria (follow from G0)

| # | PS-1 entry criterion | Evidence | Depends on (ADG) | State |
|---|---|---|---|---|
| 1 | owner-confirmed decisions for ACR-001, ACR-003, ACR-004 A and D, OD-R1, OD-R2, OD-R6, D-R8, D-K, D-T, D-C, D-0 | registry entries | GOV-01 | DONE (DEC-055..DEC-067) |
| 2 | P0-alpha Owner-Approved; PC-1 registered Provisional | registry entries | GOV-02 | DONE (DEC-071, DEC-075) |
| 3 | Core Agent, Integration Owner, Agent I named | ownership table | GOV-03 | DONE (DEC-072) |
| 4 | docs-only baseline on master | git tree | GOV-04 | DONE (dev baseline) |
| 5 | CI specification Owner-Approved and CI running as required checks | approved doc; run URLs; protection settings | GOV-05, GOV-06, XT-01, TEST-01 | DONE (required checks active) |
| 6 | ADG adopted | registry entry | GOV-02 | PARTIAL (ADG patch applied, `adg_check --strict` PASS, DEC-074; adoption entry = DEC-074) |

## 4. Not required for PS-0 or PS-1

Physical devices and Android emulator runs; the Geometry Processing Specification (needed for G2); the Voxel Architecture (needed for RG-1 and formats); owners of WorldGen, networking, server, client and engine adapter (needed from PS-11, and for the spike tasks SPK-NET / WGEN-01 when they start); GDD/TDD.

## 5. Reading the result

* **PS-0 complete** only when every row of section 1 is DONE with evidence and every decision of section 2 has an owner-confirmed entry.
* **Plan approval is different from PS-0 completion:** the owner may approve this document set (the verdict READY FOR OWNER APPROVAL means the documents are internally consistent and checked) while PS-0 itself stays open until the rows above are done.
* Current state (2026-11-09): **8 of 8 gate tasks DONE; 15 of 15 GOV-01 decisions registered (delegated); PS-0 CLOSED — G0 PASSED.** PS-1 implementation started on `prod/ps1-core-types` (CORE-01, CORE-02).

## 6. PS-1 exit criteria (plan section 15), evaluation of 2026-11-09

| # | Criterion | Evidence | State |
|---|---|---|---|
| 1 | CORE-01...CORE-11 merged; G1 members merged | `crates/cube-core` on `dev`; TEST-02 (`cube-testkit::golden`), TEST-05 (`tests/loom_gate.rs`), TEST-06 (`tests/mutation_boundary.rs`, `tests/ui/*`), XT-02 (`xtask/src/archcheck.rs`), INT-01 (DEC-076) | MET |
| 2 | Contract tests for P0-alpha and PC-1 slices pass on the CI host | `cargo test --workspace` (130+ tests) on ubuntu and windows in the required `test` jobs | MET |
| 3 | Property tests (atomicity, snapshot isolation, monotonic revisions, negative-coordinate symmetry), loom tests, reproducible goldens | `tests/commit.rs`, `tests/coordinates.rs`, `tests/loom_gate.rs` (CI step on ubuntu), goldens pinned in `revision.rs` and `tests/views.rs` | MET (goldens are produced by this implementation, not by an external reference) |
| 4 | Negative tests prove the mutation boundary | 6 trybuild compile-fail cases, commit-log completeness test, archcheck seeded violations | MET |
| 5 | Public-API snapshot published and enforced | `crates/cube-core/api-snapshot.txt`, `tests/api_snapshot.rs` | MET (text-based snapshot, not rustdoc JSON) |
| 6 | INT-01 recorded; no WorldGen/Persistence/History/Processing/Editing task, no world identity types | DEC-076; CORE-16 not implemented | MET |
| 7 | No `godot`, I/O, clocks or randomness in `cube-core` | archcheck rules ARCH-ENGINE-*, ARCH-CORE-DEPS, ARCH-CORE-IO with seeded self-tests; zero dependencies (loom only under `cfg(loom)`) | MET |

**G1 verdict:** all seven criteria are met at the commit that carries this section; known open parts of individual contracts are listed in `docs/api-contracts/README.md` and do not belong to PS-1 scope. PS-2 may start under the rules of the ADG (task entry criteria still apply, including the Geometry Processing Specification question ODM-02).

## 7. PS-2 / G2 evaluation, 2026-11-09 (DEC-081)

Statuses are kept apart: **IMPLEMENTED**, **TESTED** (CI host), **INTEGRATED** (on `dev` through the required checks), **GATE PASSED**.

| Member | IMPLEMENTED | TESTED | INTEGRATED | Evidence |
|---|---|---|---|---|
| CORE-16 | yes | yes | yes | `b50bdd0`, CI 42619486538 |
| CORE-15 | yes | yes | yes | `377ad25`, CI 42624250865; C-05 approved by DEC-079 (GOV-16) |
| PROC-01...05 | yes | yes | yes | `4436df3`, CI 42621940285 |
| PROC-06 | yes | yes | yes | `d164d21`, CI 42624528085; OD-R4 policy DEC-080 (GOV-24); per-snapshot, per-asset, active-snapshot and lifetime budgets, named refusals |
| TEST-08 | yes | yes | yes | stress (both OS) and loom models (ubuntu) in CI |
| GOV-07 | n/a | n/a | n/a | DONE through the owner-recorded alternative DEC-078 (allowed by the GOV-07 acceptance text and the PS-2 stage description); PR-01...PR-08 are **not** Owner-Approved or Frozen; RG-2 stays mandatory |
| GOV-16 | n/a | n/a | n/a | DONE, DEC-079 |
| GOV-24 | n/a | n/a | n/a | DONE, DEC-080 |

**Exit criteria checked** (not only the existence of code): snapshot isolation and apron consistency (`processing_snapshot.rs`); provider DAG with deterministic order and cycle diagnostics (`graph.rs`, random-graph property test); cancellation at every state (`processing_executor.rs`, `processing_caps.rs`); stale rejection under explored interleavings (loom: slot, token, executor racing a commit; stress without violations); resource caps on the synthetic mobile budget; bounds integration (`BoundsProvider`: unknown data is a failure, never Empty); no direct Core mutation from Processing (archcheck ARCH-HANDLE and the crate graph); compliance with the provisional limits (`processing-foundation-contract.md`). Tools on the integration commit `d164d21`: `cargo fmt`, `clippy -D warnings`, `cargo test --workspace`, `cargo deny`, `cargo xtask archcheck` (all required checks green in CI run 42624528085), `docs/audit/adg_check.py --strict` PASS.

**PS-2 CLOSED - G2 PASSED** (conditional on RG-2: the provisional Processing contracts are diffed against the Geometry Processing Specification when it exists, before PR-01...PR-08 are frozen). PS-3 may start under the ADG.

## 8. PS-3 / G3 evaluation, 2026-11-09 (DEC-087)

| Member | IMPLEMENTED | TESTED | INTEGRATED | Evidence |
|---|---|---|---|---|
| API-01..API-05 | yes | yes (12 mock contract tests, validation and state-table unit tests, API snapshot) | yes | `1f588c9`, CI 42626817723; journal phases and `AppliedUnconfirmed` added at close (change request of CB-1) |
| GOV-09 | n/a | n/a | n/a | DONE: DEC-082 (contracts, seven provisional resolutions), DEC-083 (permission schema owner), DEC-084 (stroke = one transaction), DEC-085 (delta/checkpoint deferred to PS-10), DEC-086 (journal ownership, observer delivery, blocker CB-1) |

**Exit criteria checked:** contract tests and negative cases (authorization denial leaves no state, no extra rights from `LocalClient`, illegal state transitions, malformed operations); public-API snapshot regenerated for the CB-1 change; no behaviour in `cube-editing-api` beyond construction checks and table lookups (it contains only types, traits and the origin mapping); dependency boundary (`cube-editing-api -> cube-core, cube-processing`, archcheck); compatibility with Core and Processing (builds against both, uses `Snapshot`, `RevisionToken`, `WriteSet`, `AssetView`); no unresolved merge conflict markers anywhere in the tree (new archcheck rule ARCH-CONFLICT-MARKER with seeded self-tests, runs in CI). Contracts are not Frozen.

**PS-3 CLOSED - G3 PASSED.**

## 9. PS-4 / G4 evaluation, 2026-11-09 (DEC-088)

| Member | IMPLEMENTED | TESTED | INTEGRATED | Evidence |
|---|---|---|---|---|
| ORCH-01 | yes (`txn.rs`) | yes: lifecycle, rollback/cancel, atomic failure, model-based random sessions | yes | `tests/orchestration_path.rs`, CI run 42630261984 on `920f632` |
| ORCH-02 | yes (`registry.rs`) | yes: dispatch matrix (unsupported kind, missing backend, mixed types, duplicates) | yes | `orchestration_path.rs`, `backend_conformance.rs` |
| ORCH-03 | yes (`validate.rs`) | yes: conflict matrix (a..e), exact conflicting keys, retry by OperationId, id reuse | yes | `orchestration_path.rs` |
| ORCH-04 | yes (`dirty.rs`) | yes: sink-then-observers order, post-commit state only, isolated observer failure | yes | `orchestration_path.rs`, `orchestration_durability.rs` (order equals Core commit order under 6 writers) |
| ORCH-05 | yes (`commit_path.rs`, `auth.rs`) | yes: authorization negatives, no extra rights, system origin refused, three-writer audit | yes | `orchestration_path.rs`, `mutation_negative.rs`; archcheck rule ARCH-USER-COMMIT-PATH with seeded self-test |
| TEST-03a | yes (`cube-integration-tests/src/conformance/backend.rs`) | yes: reference backend and a conforming fake pass, nine broken fakes are each caught by their rule | yes | `backend_conformance.rs` |
| TEST-10 | yes | yes: five compile-fail cases (forging a user or system handle, calling the crate-private commit, taking the handle out of the service, an observer committing) plus runtime negatives | yes | `mutation_negative.rs`, `tests/ui_mutation/*` |

**Real path, not mocks of the path:** every test above runs a real Core asset, the real `OrchestrationService` and the real commit gate. Only what lies outside PS-4 is a fake: a reference backend for the voxel representation (real backends are PS-5, not G4 members), a journal with failure injection (the durable journal is PS-10, so no crash durability is claimed; the CB-1 boundary of DEC-086 is tested instead) and recording observers.

**Acceptance criterion "spike `cube-editing` behaviours reproduced":** the spike code is not in the production base (ADG decision D-5). The behaviours it stood for (set/add/remove material and geometry, atomic multi-brick edits, conflict on concurrent change, rollback) are reproduced through the single user path by the tests above.

**Three-writer audit:** the commit log of one asset shows `User` (only through Orchestration), `System` (`SystemCommit::materialise`) and `Recovery` (`RecoveryCommit::restore`) as distinct actors; 90 randomised requests with every origin never produce anything but `User`; a `System`-origin request is refused, not downgraded.

**Known limits (not hidden):** `GeometryDelta` handed to observers carries tokens and dirty regions but no brick payloads (History/Persistence stages); one service serves one asset; the retry table is bounded (default 1024 ids); observers run inside the commit section and must not call back into the service.

**PS-4 CLOSED - G4 PASSED** (all seven members merged, required CI green, real-path mutation tests and backend conformance pass, no bypass of the commit boundary).

## 10. PS-5 / G5 status, 2026-11-09 (in progress; G5 NOT passed)

| Task | IMPLEMENTED | TESTED | INTEGRATED | Notes |
|---|---|---|---|---|
| VOX-01 | yes (`cube-backend-voxel/src/{ops,edit,params}.rs`) | yes: full chain Editing API -> Orchestration -> VoxelBackend -> Core (add, replace, remove, atomic multi-brick, brick borders, negative and extreme coordinates, micro levels, conflicts, permission denial, failed operations change nothing, unloaded bricks never guessed, other channels preserved, brute-force model for set/replace/copy/move, conformance suite) | yes (`bf8d244`, CI 42633356471) | no direct commit: the crate returns write sets |
| VOX-02 | yes (`faces.rs`, provisional: provider ownership O-1 open) | yes: brute-force definition equality across borders and negative space, border hidden/visible cases, unloaded neighbour flag, unloaded centre = failure, determinism | yes (`bf8d244`, CI 42633356471) | |
| VOX-03 | yes (`greedy.rs`, provisional) | yes: coverage equals faces exactly, seam rule (unit ring), merging and material splits, determinism for 1 and 4 workers | yes (`bf8d244`, CI 42633356471) | **limit:** T-junctions inside a brick between interior quads and the ring are not claimed fixed; greedy is not the default renderer (needs GOV-13 and measurements) |
| VOX-04 | yes (`sections.rs`: brick plus six face neighbours) | yes: incremental rebuild by the rule equals full rebuild over random edits; stale rejection after a Core commit through the dirty sink | yes (`bf8d244`, CI 42633356471) | |
| VOX-05 | yes (`collision.rs`, `lod.rs`; O-1 resolved by DEC-089) | yes: see section 11 | yes (`baaff7f`, CI 42636955092) | |
| CORE-12, GOV-08, GOV-17 | **not started / owner** | - | - | C-10 needs the Voxel Architecture revalidation (external input EXT-VA-RG1); GOV-17 is an owner decision after RG-1 |
| SDF-01..03, MSH-01..03 | **not started** | - | - | independent of the voxel work; to follow |

**G5: NOT passed.** Open: VOX-05, SDF-01..03, MSH-01..03, CORE-12, GOV-08, GOV-17 (EXT-VA-RG1 is an external input of the stage).

**Real now:** a user edit of voxel content works through `Editing API -> Orchestration -> Voxel Backend -> Core commit`, with derived face and greedy meshes recomputed per brick by Processing and invalidated by the commit. No crash durability is claimed (PS-10; CB-1 boundary unchanged).

**Technical debt registered (not refactored without a contract):**

* TD-1, observers inside the commit section: they run there so that their order equals the commit order. Moving ordinary observers out needs a per-asset ordered delivery queue (sequence numbers, a single drain, isolation kept, durable effects staying inside). A safe change must be covered by loom models of the queue; it needs no contract change but is deferred until a slow observer is a real problem.
* TD-2, the `GeometryBackend::validate` port carries a region but no layer; the voxel backend therefore reports `validate: false`. Needs an additive port change.
* TD-3, `PERS-01` (codec) is not started; `TEST-04` is implemented (see below).


**TEST-04 (independent, PS-10 prerequisite):** `cube-testkit::faultfs` is a fault-injecting in-memory filesystem with a weakest-POSIX model (data volatile until `fsync`, names volatile until `fsync_dir`, atomic `rename`, torn tails) and a crash-point runner `explore` that crashes a scenario after every numbered step under three torn-write modes. Self-tests show that the textbook replace protocol survives every point and that a missing `fsync` and an in-place overwrite are each caught. It makes no claim about real platforms (DG-2). INTEGRATED (`bf8d244`); PERS-01 and the durable journal remain PS-10 work.

## 11. Session 6, 2026-11-09: VOX-05, O-1, Persistence foundation, RG-1

| Item | IMPLEMENTED | TESTED | INTEGRATED | Notes |
|---|---|---|---|---|
| O-1 | n/a | n/a | n/a | DEC-089 assigns technical owners inside the existing agent roles; no owner decision needed (reversible, DEC-072 deferral resolved) |
| VOX-05 | yes | yes: collision boxes are disjoint and cover exactly the solid voxels (brute-force model, 10 seeds, border and negative space), unloaded = failure (never free space), stale after an edit and never served, LOD equals brute-force coarsening and never loses presence, Core hash unchanged, determinism for 0/1/4 workers | yes (`baaff7f`, CI 42636955092) | one coarse level per brick; mixed-resolution neighbours are PS-9 |
| PERS-01 | yes (`cube-persistence/src/codec.rs`) | yes: round trip, deterministic bytes, CRC check value, every single-byte flip and every truncation detected, hostile counts with a valid checksum refused without allocation, schema/kind/limit checks | yes (`baaff7f`, CI 42636955092) | **experimental** schema (bit 0x8000); GOV-10 open; not a stable format |
| Atomic snapshot files, journal reconciliation | yes (`store.rs`, `recovery.rs`) | yes: create -> edit -> save -> load -> continue on a real directory; every crash point of the model leaves a complete old or new snapshot; the explorer finds a protocol without fsync; injected failures at every step; pending write-ahead intents resolved against Core's commit log (Confirm, Abort, Undecidable) | yes (`baaff7f`, CI 42636955092) | model and real-directory evidence only; **no device durability is claimed (DG-2)** |
| RG-1 / GOV-08 / GOV-17 / CORE-12 | not started | - | - | see below |
| SDF-01..03, MSH-01..03 | not started | - | - | not reached in this session; independent of the open decisions |

**Known gap (found while testing the save/load path):** the retry table of Orchestration lives in memory. After a restart, a client that retries an operation id applied before the restart would be applied again. Idempotency across restarts needs the durable journal (PS-10): it must be rebuilt from `Confirmed` records on start. Registered as TD-4; it does not affect single-session correctness.

**RG-1 analysis (no new document needed):** the plan makes RG-1 an external-input gate (`EXT-VA-RG1`); it blocks the Frozen status of PC-1, CORE-12, CORE-13 approval and GOV-10. The plan has no built-in provisional alternative for RG-1, but GOV-07 shows the accepted pattern: an owner-recorded alternative. Minimal proposal for one owner sentence: *"For G5 only, PC-1 (C-07, C-08, C-09) is confirmed as the working baseline and C-10 is approved as the minimal provisional palette contract implemented by CORE-12; the Voxel Architecture revalidation (RG-1) stays mandatory before any contract is Frozen and before GOV-10; no save format is approved."* Without that sentence, GOV-08, GOV-17 and CORE-12 stay open and **G5 cannot close**; nothing was recorded on the owner's behalf.


## 12. Session 7, 2026-11-09: TD-4, CORE-12 (provisional)

| Item | IMPLEMENTED | TESTED | INTEGRATED | Notes |
|---|---|---|---|---|
| TD-4 (idempotency across restart) | yes: `recover_idempotency`, `resolve_pending`, bounded table with per-scope watermark, member fingerprints in journal records (additive API change, snapshot updated, codec updated) | yes (`orchestration_restart.rs`): a confirmed operation is not applied again after a restart; same id with other content refused; changes lost since the snapshot are reported and re-appliable, never falsely confirmed; an Intent without a later record is undecided, retries refused until resolved; crash between Core commit and Confirmed resolved as applied without re-applying; the table is bounded and forgotten ids are refused, not re-applied; fingerprint pinned | yes (`22d08c4`, CI 42640577990) | **Partly fixed, not closed:** see limits below |
| CORE-12 / C-10 (palette) | yes (`a15595e`) | yes: id validation, registration order, collision diagnostics, capacity 65535, table round trip, permuted-order mapping, strict resolution lists all unknown content, recovery placeholders keep original ids, air reserved, Core content guard still green; CI 42641169963 | **no (not merged)**: ADG merge-after GOV-17; the owner allowed development on a feature branch only | provisional; no binary format; RG-1 stays mandatory |
| SDF-01..03, MSH-01..03 | not started | - | - | not reached in this session |
| PERS-02..05 | not started | - | - | next after TD-4; formats stay experimental until GOV-10 |

**TD-4 limits (stated, not hidden):**

* The journal carries no write payload, so a change confirmed *after* the last snapshot is lost if the process dies; recovery reports it (`lost_since_snapshot`) and lets the client re-apply it, but cannot replay it. Real replay needs the batch in the journal (PERS-04/05, PS-10).
* Deciding an undecided Intent (applied or not) is left to the caller of `resolve_pending`; the procedure that compares journal and restored world belongs to Persistence (PS-10).
* A recovered success carries the journalled tokens but no dirty regions (they are not journalled).
* The retry window relies on operation sequences increasing per scope; a client that reuses low sequences after the window moved is refused.
* The journal here is a contract fake; no file durability is claimed (DG-2 and the PS-10 gates stay).
* **CB-1 stays open:** Core commit and the journal are still not one atomic action; the write-ahead phases plus the procedures above bound the damage but do not make it crash-atomic.

**RG-1 / GOV-08 / GOV-17:** the owner's permission covers developing CORE-12 provisionally, which is done on its branch. It does not close GOV-08/GOV-17 or RG-1, and the ADG merge condition of CORE-12 (GOV-17) is unchanged, so **G5 remains open** (also SDF-01..03 and MSH-01..03 are not done). Minimal owner record that would unblock the merge: *"GOV-17: C-10 is approved as the minimal provisional palette contract as implemented by CORE-12 (branch `prod/ps5-core12-palette`); GOV-08: PC-1 is confirmed as the working baseline for G5; RG-1 stays mandatory before any contract is Frozen and before GOV-10."*

## 13. Session 8, 2026-11-09: GOV-08, GOV-17 and the CORE-12 exception (DEC-090)

The owner recorded GOV-08 (PC-1 as the provisional baseline for PS-5) and GOV-17 (C-10 palette part approved as provisional), and allowed CORE-12 to be integrated. The ADG merge condition `EXT-VA-RG1` of CORE-12 is satisfied for that one merge by the owner-recorded exception of DEC-090. **RG-1 is not passed, EXT-VA-RG1 is not supplied, G5 is not passed.** Conformance check of CORE-12 against C-10: the implemented palette part (palette entries, unresolved-reference error, width boundaries) matches; `LayerMapping` and the revision wiring are outside CORE-12 and stay Draft.

## 14. Session 8 results (DEC-090 and the payload journal)

| Item | IMPLEMENTED | TESTED | INTEGRATED | Notes |
|---|---|---|---|---|
| CORE-12 / C-10 palette part | yes | yes | **yes** (`718e29b`, CI 42642936047) | integrated under the DEC-090 exception from EXT-VA-RG1; provisional; RG-1 not passed |
| Write payload in the journal, batch codec, `replay_confirmed` | yes (`15456e3`) | yes (`persistence_replay.rs`, CI 42643790057): confirmed changes after the last snapshot are replayed exactly (world hash equal), replay is idempotent, then `recover_idempotency` finds all operations contained; an unresolved Intent is not replayed; damaged or missing payloads are reported; records survive the codec | see the next CI run | additive API change (`JournalEntry::payload`, `BatchEncoder`), snapshot updated |
| TD-4 | loss part now closed by replay (when the journal carries payload) | yes | see above | **still not closed:** deciding an unresolved Intent stays with `resolve_pending`; the journal is a contract fake; crash durability of real files is not proven on devices; CB-1 open |
| SDF-01..03, MSH-01..03 | not started | - | - | next backend work |
| PERS-02..05 (manifest, snapshot store, durable journal file) | not started | - | - | formats stay experimental until GOV-10 |

**G5: NOT passed** (SDF, Mesh and the T-junction criterion are open; RG-1 not passed). **G10: not evaluated.**

## 15. Session 9, 2026-11-10: PERS-04 file journal and recovery guarantees

| Item | IMPLEMENTED | TESTED | INTEGRATED | Notes |
|---|---|---|---|---|
| PERS-04 file journal (`journal.rs`) | yes | yes (`persistence_journal.rs`): round trip and reopen, gapless sequence, torn tail repaired, damage in the middle refused, sequence gap and unknown schema refused, write/fsync failure poisons the journal, oversize refused; **crash at every step** of real orchestration (3 torn-write modes) keeps every acknowledged change and replay restores it (the test fails when the fsync is removed: mutation checked); two restarts on real files; killed child process | see CI | model, real files, killed process; no device durability |
| PERS-02, PERS-03, PERS-05 | **not started** | - | - | start-after `SPK-PERS` (ADG Part C): no durability spike report exists, so the dependency is **not** closed. PERS-08 world record is also not started |
| TD-4 | restart idempotency, replay of changes after the last snapshot and the undecided-intent procedure are implemented and tested on real files | yes | yes (journal part after CI) | Still not closed: `resolve_pending` needs a caller that compares the restored world; no manifest ties snapshot and journal position yet |
| CB-1 | boundary behaviours proven: crash before the Intent, after the Intent, after the Core commit but before Confirmed (retry blocked, resolved explicitly), after Confirmed; acknowledged means durable | yes (model and real files) | - | **Open:** Core commit and journal are still two separate actions. Proven: no acknowledged change is lost, no id is applied twice, an undecided outcome is refused not guessed. Not covered: crash during snapshot publication and compaction (PERS-02/03/05 do not exist yet) |

Known limit: a bit-rotted **last** record cannot be told from a torn tail and is dropped on open; rot in the middle is refused.

## 16. Session 10, 2026-11-10: SPK-PERS, journal hardening, PERS-02/03/05

| Item | IMPLEMENTED | TESTED | INTEGRATED | Notes |
|---|---|---|---|---|
| SPK-PERS | yes (spike `03d1ff6`, never merged) | yes: E1..E5, CI 42719662199 | report only (`docs/research/spk-pers-durability-report.md`) | no earlier report existed; Windows observed, Linux CI assertions, mobile untested; DG-2 not answered |
| PERS-04 hardening | yes (`9b44195`): acknowledgement marks, fail-closed open | yes: corrupt/cut/missing last acknowledged record refused with no truncation; corrupt or torn unacknowledged record dropped; mid-log damage, lost or backwards marks, foreign schema refused; crash-point exploration unchanged; CI 42720021146 | yes (`dev` c87534e) | the earlier limit "a rotted last record looks like a torn tail" is **removed** for single-file damage; undetectable if marks and journal lose the same record |
| PERS-02 manifest | yes (`a875b08`) | yes | yes (`dev` c87534e) | atomic publish, generation +1, damaged manifest is an error not "no world" |
| PERS-03 snapshot store | yes | yes | yes (`dev` c87534e) | invalid until read back and hash-checked; manifest checks size and content hash again on load |
| PERS-05 compaction | yes | yes: `FaultFs` crash at every step in all torn modes leaves one whole generation, no acknowledged record lost, hash equals the independent model, no stray files; boundary refusals | yes (`dev` c87534e) | needs exclusive access; boundary must have no open operation |
| Recovery path | yes (`open_world`) | yes: real directory, world, operations through real Orchestration and voxel backend, journal, compaction, more operations, kill, load, replay, hash equal to an independent model run, same id answered, editing continues | yes (`dev` c87534e) | |
| TD-4 | window of idempotent retries is now "since the last compaction" (**L-COMP-1**, characterised by a test) | yes | - | **Still not closed:** `resolve_pending` caller, retry history across compaction |
| CB-1 | crash during snapshot publication and compaction is now covered | yes | - | **Open:** commit and journal remain two actions |
| SDF-01..03, MSH-01..03, greedy T-junctions | not started this session | - | - | G5 and G10 NOT passed |

## 17. Session 11, 2026-11-10: L-COMP-1, SDF, Mesh, WorldGen 11A, client foundation, headless first world

| Item | IMPLEMENTED | TESTED | INTEGRATED | Notes |
|---|---|---|---|---|
| L-COMP-1 (retry floors) | yes (`780d8cd`): per-scope floor in the manifest, `restore_retry_floors` | yes (`persistence_retry.rs`, `persistence_durable.rs`): two actors committing concurrently, compaction, restart, retry of old id refused without a commit, changed payload refused, second compaction, second restart, bounded table; removing the restore makes the tests fail | after merge | sound under requirement RS-1 (`editing-contracts.md`); a source with non-increasing sequences is refused, not applied |
| TD-4 | window of old retries is now "all ids at or below the floor are refused" | yes | - | **Still not closed:** `resolve_pending` caller, CB-1 (commit and journal are two actions), RS-1 is an Orchestration requirement that the Core id model (OD-R8, opaque) does not itself give |
| SDF-01..03 | yes (`c4d3ef1`, `cube-backend-sdf`) | yes (`sdf_backend.rs`, CI 42724413300): add/subtract/material, conformance, closed manifold over eight bricks with Euler characteristic 2, incremental equals full rebuild, collision covers exactly the solid voxels, normals within the documented tolerance, unloaded bricks never guessed | after merge | provisional (providers pending O-1; port validate has no layer: TD-2, so `validate_layer` is an inherent method) |
| MSH-01..03 | yes (`4a91cfc`, `cube-backend-mesh`) | yes (`mesh_backend.rs`, CI 42725377815): conformance, closed manifold over several pages, canonical pages independent of history, typed refusals, golden collision output | after merge | needed an additive Core change: mesh layers get a page store (commit gate, view, hash, Processing snapshot accept pages); API snapshot regenerated (two additions) |
| G5 | - | - | - | **NOT passed.** Members implemented: CORE-12 (DEC-090 exception), VOX-01..05, SDF-01..03, MSH-01..03, GOV-08/GOV-17 (provisional). Remaining exactly: (1) EXT-VA-RG1 / RG-1 not supplied and not replaced by an owner alternative (DEC-090 covers CORE-12 only); (2) the VOX-03 acceptance "no T-junctions" holds between bricks but not inside a brick between interior quads (limit of section 10); (3) benchmarks exist as CI-host reports only |
| WGEN-01 / TEST-09 | yes: `cube-worldgen` terrain-v1 (integer only, versioned registry) | yes (`determinism.rs`): pinned hashes, order and thread independence, voxel oracle incl. negative coordinates, continuity over brick borders, range and error codes, generation never overwrites a loaded brick | after merge | owner: DEC-091 (technical assignment, narrow). CI host only; device equality is DG-2 |
| WGEN-02..05, LEG-01 | not started | - | - | WGEN-02/03 gated by GOV-21/GOV-11; materialiser is not built (tests call the system-commit port directly) |
| CLI-00 (and engine-neutral CLI-01/02 parts) | yes: `cube-client` | yes (unit tests with a mock source; `first_world.rs` with the real executor) | after merge | read-only, no engine, no authority; CLI-03/04, ADP-01..03 not started |
| Headless first world | yes (test scenario) | yes (`first_world.rs`): seed -> generated bricks in Core -> faces -> client batches equal an independent oracle; an edit updates the geometry; kill and restore from seed plus journal gives the exact edited world hash | after merge | **not player visible**: no adapter, no window, no input |
| PERS-06..09, ITG-01, QA-P2 | not started this session | - | - | PERS-08 (world record) is the next persistence task; G10 NOT passed |
| G11, G12 | - | - | - | NOT passed (WGEN-02..05, GOV-21, CLI-01..04, ADP-01..03 open) |
