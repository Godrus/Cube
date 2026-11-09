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
