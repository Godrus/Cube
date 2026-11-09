# PROJECT STATUS: Cube

**Document status:** **Proposed** (prepared for owner review; not yet in the repository). Once the owner accepts it, it lives at `docs/PROJECT_STATUS.md` and is the **single source of the current development state**.
**State as of the last check:** 2026-11-05, audited baseline `2c0c1f3`; this documentation set is commit `60d0bfc` (see 5.2). Everything below is a **snapshot of verified facts**, not a plan. A claim without evidence in section 4 is not a fact.
**Evidence vocabulary:** Verified / Reported / Inferred / Unknown (see `Independent-Production-Audit.md`).
**Status ladder for work (never skip a rung):** documented/specified → Implemented → Verified → Accepted. A specification is not an implementation; an implementation is not verified; verified is not accepted.

---

## 5.1 Project Identity

* **Name:** Cube. Cross-platform voxel open-world RPG / MMORPG.
* **Pillars:** practically endless procedural seamless world; full modification, destruction and building of a voxel world; Universal Geometry System (voxels, micro-voxels, SDF, polygon mesh); PC, Android, iOS (incl. tablets); cross-play; single-player, co-op, multiplayer; official, private and local servers; up to 500 concurrent players per logical world with load distribution across regions/servers; hybrid RPG progression; modular architecture with independent subsystems.
* **References (inspiration only):** early Cube World, Veloren (architecture, open development, multiplayer), Trove (classes, dungeons, bosses, collections, building).
* **Recorded architectural principles (agent-transcribed owner decisions, awaiting countersignature; see section 5.5):** Rust runtime with a pure-Rust Core; Godot 4.x as the first client via GDExtension (no `godot` dependency outside the adapter); authoritative server runtime, embedded in-process for single-player and dedicated for multiplayer with identical semantics; client is never the source of truth; hybrid world generation (C3): system generation plus Editing API for user edits plus journal/snapshot persistence; spike code never moves to production without an explicit owner decision.
* **Document map**

| Topic | Location | State |
|---|---|---|
| GDD/TDD 0.3 | absent | **MISSING (blocker B-1)** |
| Voxel Architecture 1.0 | absent | **MISSING (blocker B-1)**; revalidation gate RG-1 |
| Geometry Processing Specification | absent | **MISSING (blocker B-1)**; revalidation gate RG-2 |
| Architecture Update AU-1.0 | `docs/decisions/architecture-update-v1.0.md` | reconstructed copy, not byte-verified |
| Geometry Editing Specification, Agent Spec, Task Pack, QA Pack | `docs/architecture/`, `docs/specifications/`, `docs/agent-tasks/` | present |
| Owner decisions (AOD-01…24), approval matrix, ACR-004 package | `docs/decisions/` | recorded, awaiting countersignature |
| Contracts | `docs/api-contracts/` | all **Proposed** (none Owner-Approved) |
| Architecture roadmap and reference study | `docs/architecture/roadmap-v1.md`, `system-boundaries-v1.md`, `docs/research/` | Proposed |
| **Production Plan v2.1** | `docs/production/` (9 files) | **Proposed** |
| Release path | `DEVELOPMENT_ROADMAP.md`, `RELEASE_CRITERIA.md` (this package) | Proposed |
| Agent entry | `AGENT_HANDOFF.md` (this package) | Proposed |

## 5.2 Repository Baseline

| Field | Value | Class |
|---|---|---|
| Branch | `docs/architecture-roadmap-v1` | Verified |
| **Last independently verified baseline** | `2c0c1f37b2ac5f56a37b0995b6b735ab0b04f3e0` (Production Plan v2.1). The audit of the plan and repository refers to this commit | Verified |
| **Commit holding this status documentation** | `60d0bfc837f135b48e7e86163de8bf9eeb2b2a1f`; `2c0c1f3..60d0bfc` verified: 8 files, +920/-0, docs only, no production, spike or configuration change | Verified |
| **Current HEAD** | **not recorded here by design** (a file cannot contain the hash of the commit that contains it). Read it with `git rev-parse HEAD`. Any commit after `60d0bfc` must be inspected with `git diff 60d0bfc..HEAD --stat` before this file is trusted | rule |
| Pushed | `origin/docs/architecture-roadmap-v1` equals HEAD | Verified |
| Working tree | **not clean:** `.gitignore` modified (adds `AGENTS.md`); this is the owner's own change and is not part of any commit | Verified |
| Production code | **none** (no `xtask`, no `cube-editing-api`, no `cube-integration-tests`, no toolchain pin, no `deny.toml`) | Verified |
| Spike code | present on this branch: 7 crates, ~5 900 Rust lines, `godot-client/`; 131 tests pass (desktop, experimental; never for `master`) | Verified |
| CI | **none** (`.github/` does not exist) | Verified |
| Required status checks on GitHub | cannot be read (`gh` missing); effectively none because no workflow exists | Unknown / Inferred |
| archcheck | not implemented; only the spike manifest test `crates/cube-core/tests/dependency_rules.rs` exists, which cannot detect mutation-boundary bypasses | Verified |
| `master` | 13 files; docs branch is 33 commits / 259 files ahead | Verified |
| Date of check | 2026-11-05 | |

## 5.3 Current Development Stage

| Question | Answer |
|---|---|
| Release milestone | **M0 Architecture & Governance: in progress, not reached.** No milestone M0…M8 is reached (`DEVELOPMENT_ROADMAP.md`). |
| Project phase | **Architecture & governance preparation. No production implementation exists.** Documentation preparation, architecture approval and implementation are separate phases; only the first is under way. |
| Production Stage | **PS-0 Governance and Architecture Freeze: open, 0 of 8 gate tasks done.** (PS-1…PS-14 not started.) |
| Current Gate | **G0** (not satisfied). Satisfied gates: none. |
| Current task | **GOV-01**: the owner's decision package (ACR-001/002/003, ACR-004 part A, OAM-10 principle, OD-R1/R2/R6, D-R8, D-K, D-T, D-C, D-0, D-E, branch option). |
| Task status | **Ready** (its dependency list is empty), **awaiting the project owner**. Nothing has been decided in a registry. |
| Next permitted task | `GOV-02` (adopt P0-alpha, PC-1, status model) after GOV-01; `GOV-03` (owners) and `DEV-01` and `SPK-PERS` are independent and also startable. Engineering tasks of PS-1 are **not** permitted. |
| What blocks the transition PS-0 → PS-1 | `G0` unsatisfied: GOV-01…06, TEST-01, XT-01 all Not Started; no registry, no CI, no archcheck, no baseline branch. |
| Who unblocks | **Project owner** for GOV-01/02/03 and the branch option; then the Integration Owner (GOV-04/05, XT-01) and Agent I (GOV-06, TEST-01). |

## 5.4 Development Progress

Task counts come from the ADG in `module-boundaries.md` Part C/D. **Statuses:** *Not Started* means no artefact in the repository. No task of the 147 has reached Implemented, Verified or Accepted. Those words may be used only with evidence recorded in section 4.

| Stage / Gate | Name | Status | Done | Remaining | Acceptance (closing condition, ADG Part D) | Evidence | Blockers | Dependencies (start-after / merge-after) | To pass on |
|---|---|---|---|---|---|---|---|---|---|
| PS-0 / G0 | Governance and Architecture Freeze | **Open** (current stage) | 0 of 8 gate tasks (+3 outside gate: DEV-01, SPK-NET, SPK-PERS) | 8 gate tasks: GOV-01, GOV-02, GOV-03, GOV-04, GOV-05, GOV-06, TEST-01, XT-01 | all 8 member tasks merged and accepted; CI specification Owner-Approved and, as a separate condition, CI running (workflow-run URL, required-status-check settings); registry entries owner-confirmed | none | BLK-01,02,03,07,09 | S: -; M: - | G0 satisfied |
| PS-1 / G1 | Geometry Core Foundation (P0-alpha) | Not Started | 0 of 16 gate tasks | 16 gate tasks: CORE-01, CORE-02, CORE-03, CORE-04, CORE-05, CORE-06, CORE-07, CORE-08, CORE-09, CORE-10, CORE-11, TEST-02, TEST-05, XT-02, TEST-06, INT-01 | all 16 member tasks merged and accepted; P0-alpha Owner-Approved (done in PS-0) and recorded Implemented by INT-01; PC-1 stays under Provisional approval; no contract Frozen; CI-host evidence only - no device, no emulator condition | none | G0 unsatisfied | S: G0, TEST-01; M: - | G1 satisfied |
| PS-2 / G2 | Processing Foundation | **Closed - G2 PASSED** (DEC-081; RG-2 revalidation stays open) | 12 of 12 gate tasks | 12 gate tasks: CORE-15, PROC-01, PROC-02, PROC-03, PROC-04, PROC-05, PROC-06, TEST-08, GOV-07, CORE-16, GOV-16, GOV-24 | all 12 member tasks merged and accepted; includes GOV-07: PR-01…PR-08 Owner-Approved after RG-2; development may precede it, production merges that depend on Processing may not | none | BLK-04 (EXT-GPS), G1 intent (D-01) | S: CORE-10; M: CORE-10, CORE-06 | G2 satisfied |
| PS-3 / G3 | Editing API and Contracts | **Closed - G3 PASSED** (DEC-087) | 6 of 6 gate tasks | 6 gate tasks: API-01, API-02, API-03, API-04, API-05, GOV-09 | all 6 member tasks merged and accepted; E-02…E-12 and the storage ports Owner-Approved (GOV-09); API-02 merged only after G2 | none | G2 | S: CORE-10, PROC-01, PROC-04, CORE-09; M: CORE-07, G2 | G3 satisfied |
| PS-4 / G4 | Editing Orchestration | **Closed - G4 PASSED** (DEC-088) | 7 of 7 gate tasks | 7 gate tasks: ORCH-01, ORCH-02, ORCH-03, ORCH-04, ORCH-05, TEST-03a, TEST-10 | all 7 member tasks merged and accepted; Orchestration merged only after G2 and G3 | none | G2, G3 | S: API-02, CORE-10, API-01, PROC-02, API-04, API-03, CORE-08; M: G2, G3, XT-02, API-02 | G4 satisfied |
| PS-5 / G5 | Geometry Backends and Derived Providers | Not Started | 0 of 14 gate tasks | 14 gate tasks: CORE-12, VOX-01, VOX-02, VOX-03, VOX-04, VOX-05, SDF-01, SDF-02, SDF-03, MSH-01, MSH-02, MSH-03, GOV-08, GOV-17 | all 14 member tasks merged and accepted; backend conformance only; renderer validation is DG-1 and the renderer default is GOV-13, both outside this stage | none | BLK-05 (RG-1), G2, G3 | S: CORE-10, API-02, TEST-03a, PROC-01, PROC-02, CORE-03; M: CORE-10, EXT-VA-RG1, G2, G3 | G5 satisfied |
| PS-6 / G6 | Tools: Sculpting and Boolean | Not Started | 0 of 7 gate tasks | 7 gate tasks: SCU-01, SCU-02, SCU-03, BOO-01, BOO-02, BOO-03, GOV-20 | all 7 member tasks merged and accepted; tool-level contracts Owner-Approved (GOV-20) | none | G4, G5 | S: API-01, ORCH-02, VOX-01, SDF-01; M: API-01, G4, G5 | G6 satisfied |
| PS-7 / G7 | History | Not Started | 0 of 5 gate tasks | 5 gate tasks: HIST-01, HIST-02, HIST-03, HIST-04, GOV-25 | all 5 member tasks merged and accepted; OD-R5 decided (GOV-25); system and recovery commits absent from history (tests) | none | G4; OD-R5 (GOV-25) | S: API-02, API-04, ORCH-03; M: G4 | G7 satisfied |
| PS-8 / G8 | Composite Geometry | Not Started | 0 of 6 gate tasks | 6 gate tasks: CORE-14, COMP-01, COMP-02, COMP-03, GOV-19, GOV-23 | all 6 member tasks merged and accepted; C-13…C-15 and composite-operation contracts Owner-Approved | none | G4; GOV-19/23 | S: CORE-10, ORCH-01, CORE-04; M: CORE-10, G4 | G8 satisfied |
| PS-9 / G9 | Resampling and Resolution Refinement | Not Started | 0 of 5 gate tasks | 5 gate tasks: CORE-13, RES-01, RES-02, RES-03, GOV-18 | all 5 member tasks merged and accepted; C-11, C-12 Owner-Approved after RG-1 (GOV-18) | none | BLK-05, G4, G5 | S: CORE-10, VOX-01, VOX-03; M: CORE-10, EXT-VA-RG1, G4, G5, GOV-23 | G9 satisfied |
| PS-10 / G10 | Persistence | Not Started | 0 of 14 gate tasks | 14 gate tasks: TEST-04, PERS-01, PERS-02, PERS-03, PERS-04, PERS-05, PERS-06, PERS-07, PERS-08, PERS-09, ITG-01, QA-P2, GOV-10, GOV-11 | all 14 member tasks merged and accepted; persistence format Owner-Approved (GOV-10, needs RG-1); desktop crash matrix green; device durability is DG-2, not a condition | none | BLK-05, SPK-PERS, GOV-10/11 | S: TEST-01, CORE-11, API-04, SPK-PERS, CORE-07, CORE-05, CORE-09, CORE-16, CORE-14, ORCH-04; M: TEST-01, CORE-11, CORE-09, CORE-16, CORE-14, G4 | G10 satisfied |
| PS-11 / G11 | World Generation | Not Started | 0 of 9 gate tasks | 9 gate tasks: WGEN-01, TEST-09, WGEN-02, WGEN-03, WGEN-04, WGEN-05, ITG-02, LEG-01, GOV-21 | all 9 member tasks merged and accepted; WG-01…WG-10 Owner-Approved (GOV-21); end-to-end test ITG-02; no direct commit left in worldgen (archcheck) | none | BLK-07 (owner), G10, GOV-11/21 | S: CORE-10, CORE-07, CORE-16, TEST-02, PROC-03, PROC-04, CORE-09, GOV-11, API-04, PERS-06; M: CORE-10, CORE-16, PROC-04, CORE-09, GOV-11, G10, PERS-08 | G11 satisfied |
| PS-12 / G12 | Client Runtime and Engine Adapter | Not Started | 0 of 10 gate tasks | 10 gate tasks: CLI-00, CLI-01, CLI-02, CLI-03, CLI-04, ADP-01, ADP-02, ADP-03, LEG-02, RND-01 | all 10 member tasks merged and accepted; desktop evidence only; no write path or history in the adapter (archcheck, TEST-10) | none | BLK-07 (owner), ORCH/HIST | S: CORE-10, API-01, CORE-06, PROC-03, VOX-02, VOX-05, GOV-11, VOX-03; M: CORE-06, VOX-03, PROC-05, VOX-05, API-01, CORE-12, ORCH-05, HIST-02, TEST-10 | G12 satisfied |
| PS-13 / G13 | Server Runtime and Networking | Not Started | 0 of 17 gate tasks | 17 gate tasks: NET-01, NET-02, NET-03, GOV-12, NET-04, SRV-01, SRV-02, SRV-03, SRV-04, SRV-05, SRV-06, SRV-07, CLI-05, TEST-03b, TEST-07, GOV-22, SRV-08 | all 17 member tasks merged and accepted; protocol, transport and server contracts Owner-Approved (GOV-12, GOV-22); conformance on both transports and both hosts | none | BLK-07, G10, GOV-12/22; SPK-NET | S: API-02, CORE-07, CORE-10, API-03, WGEN-04, ORCH-03, HIST-04, CLI-01; M: API-02, G4, G10, WGEN-03, PERS-06, ORCH-05, WGEN-04, HIST-04, CLI-01 | G13 satisfied |
| PS-14 / G14 | Full QA and Release Validation | Not Started | 0 of 8 gate tasks | 8 gate tasks: QA-02, QA-04, QA-06, DG-1, GOV-13, DG-2, DG-3, GOV-15 | all 8 member tasks merged and accepted; DG-1…DG-3 reports; renderer default recorded (GOV-13); PC-1 frozen only through GOV-15 after GOV-08 | none | BLK-10 (EXT-DEV), G13 | S: TEST-01, G13, ADP-01, VOX-03, CLI-02, EXT-DEV, WGEN-05, PERS-05; M: TEST-01, G13, ADP-01, EXT-DEV | G14 satisfied |

**Tasks outside any gate:** `DEV-01` (devices; gates only DG-1…3), `SPK-PERS` (persistence durability spike; never merged), `SPK-NET` (transport spike; owner unassigned).

**Stage transition rule (proposed, closing audit defect D-01):** a stage may be *started* when its start-after conditions hold and may *merge* only when its merge-after conditions hold **and the previous stage's gate is satisfied unless the ADG records an explicit exception**. Until the ADG is corrected (audit D-01/D-02), the agent applies this stricter rule by hand.

## 5.5 Owner Decisions

Three classes are kept apart. Nothing in B or C is a decision of the owner.

### A. Decisions recorded from the owner (agent-transcribed; owner countersignature pending)

| ID | Title | Status | Source |
|---|---|---|---|
| AOD-01…19 | Rust Core, Godot first client, GDExtension, authoritative server, client not truth, contracts stay Draft until separately approved (AOD-18), etc. | **Approved (recorded 2026-11-04 from chat); countersignature pending** | `docs/decisions/owner-architecture-decisions-v1.md` |
| AOD-20 | WorldGen to Core: hybrid C3 | Approved (recorded 2026-11-05); countersignature pending | same, section 5.1 |
| AOD-21 | Persistence: WorldSeed + GeneratorVersion, durable journal, compaction protocol | Approved; no binary format approved | section 5.2 |
| AOD-22 | Embedded in-process server + dedicated server, same runtime/semantics, conformance on both transports | Approved | section 5.3 |
| AOD-23 | Licence policy | Approved | section 5.4 |
| AOD-24 | Missing sources: temporary baseline allowed for limited parts | **Conditionally Approved** | section 5.5 |
| OAM-01/02/03/07 | Approval matrix items | Approved; OAM-08 Conditionally Approved | `docs/decisions/owner-approval-matrix-v1.md` |

### B. Decisions awaiting the owner (PS-0, in GOV-01/GOV-02): 0 of 15 confirmed

| # | ID | Question | Options | Current recommendation | Depends | Action required |
|---|---|---|---|---|---|---|
| 1 | ACR-001 | Foundation types in Core | approve / `cube-foundation` crate | approve | CORE-16…19 | owner registry entry |
| 2 | ACR-002 | Ports in L2 (`cube-editing-api`) | approve / direct dependency (rejected earlier) | approve | API-02, ORCH, backends | entry |
| 3 | ACR-003 | Composite data in Core | approve / all in Composite / all in Core | approve | CORE-14, PS-8 | entry |
| 4 | ACR-004 A | Core prerequisites (ActorKind, system-commit boundary) | approve / amend / reject | approve | CORE-07…09, PS-11 | entry |
| 5 | OAM-10 / ACR-004 D | revision model | R-A (recommended) / R-B / R-C | R-A | CORE-06, PERS, WGEN | entry |
| 6 | OD-R1 | revision token width | `u64`/`u32` / other | keep | C-16 | entry |
| 7 | OD-R2 | commit algorithm | immutable state + commit gate / cell CAS | commit gate | C-16, C-01 | entry |
| 8 | OD-R6 | region stamp | 128-bit hash / set / coarse | hash | C-16 | entry |
| 9 | D-R8 | OperationId format | composite / random / client-supplied | composite | C-17 | entry |
| 10 | D-K | key, widths, brick edge | i8/i32/16 / alternatives | as proposed | C-07 (Provisional) | entry |
| 11 | D-T | transforms | integer / f64 | integer | C-04 | entry |
| 12 | D-C | channel model | built-in / alternatives | built-in | C-08/09 (Provisional) | entry |
| 13 | D-0 | P0 scope | P0-alpha + PC-1 / wait / full P0 on baseline (contradicts AOD-24) | P0-alpha + PC-1 | all of PS-1 | entry |
| 14 | D-E | toolchain, execution model, licence allow-list | pinned stable, no async in lower layers / alternatives | as proposed | CI, all crates | entry |
| 15 | branch option | docs-only baseline | new docs-only branch / path-limited PRs / accept spike code | new docs-only branch | GOV-04 | entry |

**Source of B:** `docs/production/ps-0-exit-checklist.md` section 2; options in `architecture-blocker-decisions.md` section 1. **Consequence of inaction:** G0 stays open; no production task can start.

### C. Later decisions (Proposed; each has an owning GOV task)

GOV-03 owners (WorldGen, Networking Adapter, Server Runtime, Client Runtime, Engine Adapter, Import/Export, O-1, O-2); GOV-07 PR-01…PR-08 (after RG-2); GOV-08 RG-1 record; GOV-09 E-02…E-12; GOV-10 persistence format; GOV-11 ACR-004 B/C/E and OAM-09 (C1 recommended); GOV-12 transport; GOV-13 renderer default; GOV-15 freeze of PC-1; GOV-16…19, 23 deferred Core/composite contracts; GOV-20 tool contracts; GOV-21 WG-01…10; GOV-22 SR-01…10; GOV-24 OD-R4; GOV-25 OD-R5.

### D. New decision requested by this audit

| ID | Question | Options | Recommendation |
|---|---|---|---|
| ST-1 | Where does the canonical project state live, given `AGENTS.md` mandates `.agent/state/CURRENT_CONTEXT.md` (absent) and `AGENTS.md` is about to be git-ignored? | (a) `docs/PROJECT_STATUS.md` canonical + `.agent/state/CURRENT_CONTEXT.md` as a short pointer; (b) create `.agent/` per `AGENTS.md` and make `CURRENT_CONTEXT.md` canonical; (c) PROJECT_STATUS only, amend `AGENTS.md` | (a) |
| ST-2 | Accept audit defects D-01/D-02 as corrections to the ADG (stage-gate chain) | accept / waive a stage as optional | accept |

## 5.6 Blockers

| ID | Description | Cause | Blocks | How to remove | Who | Evidence for closing |
|---|---|---|---|---|---|---|
| BLK-01 | 15 PS-0 decisions unconfirmed | no registry, no owner entries | G0, all PS-1+ | owner confirms each in `decision-registry.md` | project owner | registry entries with owner confirmation source |
| BLK-02 | No docs-only baseline branch | docs branch carries spike code; `master` has 13 files | GOV-04, GOV-06, PS-1 | choose branch option (#15), execute GOV-04 | owner, Integration Owner | baseline PR/branch; no spike file under `crates/` |
| BLK-03 | No CI, no archcheck | none implemented | G0, every merge | GOV-05 spec Owner-Approved; GOV-06, XT-01 implemented | Integration Owner, Agent I | workflow-run URL green/red; branch protection lists required checks |
| BLK-04 | Geometry Processing Specification absent (B-1) | external input | G2 and everything after | supply spec; RG-2; GOV-07 | owner | spec in repo; GOV-07 entry |
| BLK-05 | Voxel Architecture absent (B-1) | external input | PC-1 stays Provisional; formats unapprovable; PS-5, PS-9, PS-10 merges | supply; RG-1; GOV-08 | owner | RG-1 record |
| BLK-06 | GDD/TDD absent | external input | scope beyond the plan; M3+ scope definition | supply or decide scope baseline | owner | document in repo |
| BLK-07 | Unassigned owners (WorldGen, Networking, Server, Client, Engine Adapter, ...) | no decision | PS-11…PS-13 tasks, SPK-NET | GOV-03 | owner | ownership table updated |
| BLK-08 | ADG defects D-01/D-02 | audit finding | correct scheduling of PS-2+ | correct graph, re-validate | Integration Owner, with owner acceptance (ST-2) | regenerated Part C/D, extended validator |
| BLK-09 | State-source conflict (ST-1) | `AGENTS.md` vs this package | reliable agent handoff | decide ST-1 | owner | decision entry |
| BLK-10 | Physical devices / macOS (DEV-01, NB-1/2) | not acquired | DG-1…3, G14, M5+ device claims | acquire | owner | device inventory |
| BLK-11 | GitHub settings unknown | `gh` unavailable | confidence in required checks | read-only export of branch protection | owner / agent with `gh` | JSON export attached |

## 5.7 Next Actions

**There is exactly one next task: `GOV-01`.** It is a decision of the project owner, not an engineering task.

| Field | Value |
|---|---|
| Task ID | GOV-01 |
| Title | Decide ACR-001/002/003, ACR-004 part A, OAM-10 principle, OD-R1, OD-R2, OD-R6, D-R8, D-K, D-T, D-C, D-0, D-E and the branch option |
| Owner | Project owner (the agent may only prepare and record) |
| Dependencies | none (ADG: S and I empty) |
| Preconditions | the owner has read `architecture-blocker-decisions.md` section 1 and `ps-0-exit-checklist.md` section 2; the audit defects are known |
| Allowed actions (agent) | present each decision with options/recommendation; ask the owner for a verdict; write an entry in `docs/production/decision-registry.md` **only** quoting an explicit owner statement and its source (chat date or signed commit); update this status file |
| Forbidden actions | deciding, "approving" or implying approval; setting any contract to Owner-Approved; starting any PS-1 task; creating production crates; moving spike code; committing or pushing without explicit owner permission |
| Expected result | 15 registry entries, each Approved / Amended / Rejected with the owner's confirmation source |
| Acceptance criteria | every one of the 15 decisions has an entry with decision, options considered, confirmation source and date; none inferred |
| Required tests | none (documentation); a link check that every entry cites a source and an ID from `architecture-blocker-decisions.md` |
| Evidence | the registry file in the repository, commit hash; the owner's confirmation |
| Completion condition | all 15 entries present and owner-confirmed; `GOV-01` marked Accepted here with that evidence |

**After GOV-01** (in this order of availability): GOV-02 and GOV-04 (baseline) and GOV-05 (CI specification) become startable; GOV-03 (owners), DEV-01 and SPK-PERS can start any time. PS-1 starts only when G0 is satisfied.

## 4 (evidence index)

| Claim | Evidence |
|---|---|
| Branch/HEAD/status/commit content | git commands in `Independent-Production-Audit.md` section 1 |
| 131 spike tests passing | `cargo test --workspace --offline`, 2026-11-05 |
| ADG integrity | `PROJECT-STATUS-VALIDATION.md` |
| No CI / no production code | file listing |

## 6. How this file is maintained

See `AGENT_HANDOFF.md` section "Updating status" and `RELEASE_CRITERIA.md` section 0. Every change of a status needs a row in the evidence index with a verifiable reference (command output, commit hash, workflow-run URL, registry entry).
