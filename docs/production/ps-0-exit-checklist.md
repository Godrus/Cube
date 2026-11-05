# PS-0 Exit Checklist (proposal)

**Status:** **Proposed**; a checklist, not a decision. PS-0 (Governance and Architecture Freeze) closes when gate **G0** is satisfied (ADG part C and D in `module-boundaries.md`). **Today every item is NOT DONE; PS-0 is not complete.** Marking an item DONE needs the evidence named in the row; an agent may not mark an owner-decision row DONE without the owner's registry entry (`production-architecture-plan.md` section 3.3).
**Related:** PS-1 entry criteria (section 3), decisions list (`architecture-blocker-decisions.md` section 1), verdict (`production-plan-revision-v2.1-report.md` section 2).

## 1. Gate G0 tasks

| # | Gate task | Owner | What must exist (evidence) | Verification | State |
|---|---|---|---|---|---|
| 1 | GOV-01 | Project owner | decision-registry.md entries for ACR-001, ACR-002, ACR-003, ACR-004 part A, part D principle (OAM-10), OD-R1, OD-R2, OD-R6, D-R8, D-K, D-T, D-C, D-0, D-E, the branch option; each with a confirmation source (owner-signed commit or PR approval) | Integration Owner checks every entry has a source; no entry made by an agent | NOT DONE |
| 2 | GOV-02 | Project owner | registry entries: P0-alpha Owner-Approved (C-01, C-02, C-03, C-04, C-06, C-16, C-17, C-18, C-19 incl. the ACR-004 part A amendments); PC-1 Provisional with the five mandatory fields; status model adopted; AOD-01…24 countersigned | registry diff reviewed; contracts README status column updated by the Integration Owner | NOT DONE |
| 3 | GOV-03 | Project owner | ownership table naming Core Agent, Integration Owner, Agent I and, for PS-2…PS-10, the named agents; UNASSIGNED modules listed with the stage that needs each; networking and WorldGen owners (or the spike runner) named before SPK-NET / WGEN-01 start | ownership table in module-boundaries.md matches the registry | NOT DONE |
| 4 | GOV-04 | Main Agent / Integration Owner | docs-only baseline on master (no file of the spike code on master) according to the option chosen in GOV-01 | `git ls-tree` of master lists only documentation | NOT DONE |
| 5 | GOV-05 | Main Agent / Integration Owner | CI specification Owner-Approved: checks, required statuses, tool pins, licence allow-list | Owner-Approved document | NOT DONE |
| 6 | GOV-06 | Agent I (QA and Benchmarks) | running CI: workflow-run URL green on an empty skeleton and red on seeded failures; branch protection on master lists the required checks (API output) | run URLs and protection settings attached | NOT DONE |
| 7 | TEST-01 | Agent I (QA and Benchmarks) | `cube-testkit` skeleton merged and used by at least two crates | CI run | NOT DONE |
| 8 | XT-01 | Main Agent / Integration Owner | `xtask archcheck` v0 merged; seeded violations fail; registered as a required status check | CI run with seeded failures | NOT DONE |

## 2. Decisions that GOV-01 must contain (each needs an owner-confirmed registry entry)

| # | Decision | Document | Options in short | State |
|---|---|---|---|---|
| 1 | ACR-001 foundation types in Core | blocker-decisions section 1 | approve / `cube-foundation` crate | NOT DONE |
| 2 | ACR-002 ports in L2 | blocker-decisions section 1 | approve / direct dependency (rejected earlier) | NOT DONE |
| 3 | ACR-003 composite data in Core | blocker-decisions section 1 | approve / all in Composite / all in Core | NOT DONE |
| 4 | ACR-004 part A (Core prerequisites) | blocker-decisions section 2 | approve / amend / reject | NOT DONE |
| 5 | Revision model principle (ACR-004 part D, OAM-10) | blocker-decisions section 2 | R-A (recommended) / R-B / R-C | NOT DONE |
| 6 | OD-R1 token width | blocker-decisions section 1 | `u64`/`u32` / other | NOT DONE |
| 7 | OD-R2 commit algorithm | blocker-decisions section 1 | commit gate over immutable state / cell CAS | NOT DONE |
| 8 | OD-R6 region stamp | blocker-decisions section 1 | 128-bit hash / set / coarse | NOT DONE |
| 9 | D-R8 OperationId format | blocker-decisions section 1 | composite id / random / client-supplied | NOT DONE |
| 10 | D-K key, widths, brick edge | blocker-decisions section 1 | level `i8`, `i32`, edge 16 / alternatives | NOT DONE |
| 11 | D-T transforms | blocker-decisions section 1 | integer transform / f64 | NOT DONE |
| 12 | D-C channel model | blocker-decisions section 1 | built-in channels, quantised density / alternatives | NOT DONE |
| 13 | D-0 P0 scope | blocker-decisions section 1 | P0-alpha + PC-1 / wait / full P0 on baseline (contradicts AOD-24) | NOT DONE |
| 14 | D-E toolchain, execution model, licence allow-list | blocker-decisions section 1 | pinned stable, no async runtime in lower layers / alternatives | NOT DONE |
| 15 | Branch option for the docs-only baseline | integration-and-qa-plan section 1 | new docs-only branch / path-limited PRs / accept spike code (contradicts roadmap-v1) | NOT DONE |

## 3. PS-1 entry criteria (follow from G0)

| # | PS-1 entry criterion | Evidence | Depends on (ADG) | State |
|---|---|---|---|---|
| 1 | owner-confirmed decisions for ACR-001, ACR-003, ACR-004 A and D, OD-R1, OD-R2, OD-R6, D-R8, D-K, D-T, D-C, D-0 | registry entries | GOV-01 | NOT DONE |
| 2 | P0-alpha Owner-Approved; PC-1 registered Provisional | registry entries | GOV-02 | NOT DONE |
| 3 | Core Agent, Integration Owner, Agent I named | ownership table | GOV-03 | NOT DONE |
| 4 | docs-only baseline on master | git tree | GOV-04 | NOT DONE |
| 5 | CI specification Owner-Approved and CI running as required checks | approved doc; run URLs; protection settings | GOV-05, GOV-06, XT-01, TEST-01 | NOT DONE |
| 6 | ADG adopted | registry entry | GOV-02 | NOT DONE |

## 4. Not required for PS-0 or PS-1

Physical devices and Android emulator runs; the Geometry Processing Specification (needed for G2); the Voxel Architecture (needed for RG-1 and formats); owners of WorldGen, networking, server, client and engine adapter (needed from PS-11, and for the spike tasks SPK-NET / WGEN-01 when they start); GDD/TDD.

## 5. Reading the result

* **PS-0 complete** only when every row of section 1 is DONE with evidence and every decision of section 2 has an owner-confirmed entry.
* **Plan approval is different from PS-0 completion:** the owner may approve this document set (the verdict READY FOR OWNER APPROVAL means the documents are internally consistent and checked) while PS-0 itself stays open until the rows above are done.
* Current state: **0 of 8 gate tasks DONE; 0 of 15 decisions confirmed.**
