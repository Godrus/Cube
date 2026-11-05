# Cube Production Plan Revision v2: Report

**Status:** report on a **Proposed** revision. Nothing in the seven v2 documents is approved; no contract, ACR, AOD or blocker closure changed; no code, merge, commit or branch change was made; the repository was not touched (work done in a scratchpad workspace).
**Set:** `production-architecture-plan.md`, `module-boundaries.md` (with the ADG), `implementation-roadmap.md`, `agent-task-breakdown.md`, `architecture-blocker-decisions.md`, `integration-and-qa-plan.md`, this report.

## 1. Basis and limits of this revision

* **Fact:** I did not receive the text of the "full technical audit of the six documents" or of the "second review". I treated the numbered requirements in your instruction (2.1-2.12, 3, 6) as the authoritative finding list, and added the findings of my own re-read of v1 (tagged **S** below). If the two reviews contain findings not covered by sections 3-4, send them; they are not silently assumed closed.
* The roadmap-v1 "Gate 0-10" system was taken as the "old QA Roadmap".
* The ADG, task table, stage membership and the checks of section 6 are generated and validated by a script from one data file; the script and data are in `_build/` of the workspace (not part of the proposed repository content).

## 2. Summary of corrections

1. One canonical numbering `PS-0…PS-14` with a mapping from the QA-Pack stages and the roadmap-v1 gates; bare "Stage n"/"Gate n" removed.
2. One Authoritative Dependency Graph (module grid, interface table, 131 task rows with start-after/merge-after edges, 15 gates, 3 external inputs), validated acyclic.
3. Processing moved onto the production critical path: **G2 gates every production merge of Editing and later** (ORCH, backends, tools, history, resolution, server runtime); G2 requires the Geometry Processing Specification (RG-2); parallel development starts at the Core API freeze (CORE-10).
4. Formal P0 / P0-alpha / Provisional split with a status model, decision registry and revalidation gates; D-0 contradiction removed.
5. ACR-004 split into five independent parts A-E; PS-1 closes only part A.
6. Minimal universal Core model defined (Layer, Channel, Representation, CellKey with level, Transform, Region, CommitBatch); conversions never implicit; no float in authoritative state (proposal).
7. CORE-11 (export views and content hash) placed in PS-1 and made a dependency of Persistence.
8. Single mutation boundary invariant MB-1 with capability handles, commit log, compile-fail tests and archcheck rules.
9. Persistence production requirements written down and separated from the spike; recovery no longer depends on WorldGen (new `IBaseProvider` port) or History.
10. WorldGen, networking, server, client and engine adapter concerns separated with explicit order.
11. QA fixes: no emulator/device condition for Core, Processing or Persistence; renderer validation moved to DG-1; backend conformance independent of the renderer; CI specification separate from CI implementation; concurrency, fault-injection and negative mutation tests mandatory; golden hash scope fixed.
12. Ownership matrix with the six requested columns; unassigned owners listed as owner decisions, none assigned.

## 3. Original Finding -> Correction -> Document -> Section

Finding ids `R` = your numbered requirement; `S` = my self-review of v1.

| Id | Original finding | Correction | Document | Section |
|---|---|---|---|---|
| R2.1 | three incompatible stage systems | canonical `PS-n`, mapping table, bare references removed | production-architecture-plan | 1, 2 |
| R2.1b | stage numbers inside other documents | all six documents use `PS-n`/`Gn`; re-audit checks bare references | all | re-audit (section 6) |
| R2.2 | no single dependency graph | ADG parts A-E generated and validated | module-boundaries | A-E |
| R2.2.1 | Processing not on critical path | `ORCH-*`, backends, `SRV-01` merge-after G2; G2 includes GOV-07 | module-boundaries C, D; implementation-roadmap | PS-2, PS-4 |
| R2.2.2 | start vs production gate of Processing | S edges (start after CORE-10) vs I edges (merge after G2) | module-boundaries | C, D |
| R2.2.3 | PERS-04, PERS-06 dependencies incomplete | PERS-04: API-04, CORE-07, CORE-05, PERS-01; PERS-06: PERS-05, CORE-09, API-04, GOV-11 | module-boundaries | C |
| R2.2.4 | WGEN-02 vs roadmap and persistence | WGEN-02 depends on WGEN-01, PROC-03/04 only; persistence interplay moved to WGEN-04 (storage port), ITG-02, SRV-01 | module-boundaries; implementation-roadmap | C; PS-11 |
| R2.2.5 | NET-04 and SRV-02 mix protocol types and server integration | NET-02 protocol and session types incl. handshake messages; NET-04 external transport; SRV-02 server session/auth integration | agent-task-breakdown | NET, SRV |
| R2.2.6 | Persistence deps on History, Composite, API-04, OperationId, journal | no History edge; PERS-09 uses only CORE-14; API-04 added; versioned OperationId in PERS-04; ITG-01 tests journal replay | module-boundaries | B note 2-3, C |
| R2.2.7 | cycle check | Kahn check on S, I and gate edges: acyclic | module-boundaries | E |
| R2.2.8 | task dependencies not in main graph | task tables carry no dependency column; ADG is the only source; a script rejects unknown nodes | agent-task-breakdown; module-boundaries | header; E |
| R2.3 | P0 / P0-alpha / Provisional ambiguous | definitions, rules, register PC-1, RG-1/RG-2 | production-architecture-plan | 4 |
| R2.3b | v1 D-0 contradicts the v1 Stage 1 exit | PS-1 closes on P0-alpha only; PC-1 Provisional; C-10…C-15 deferred | production-architecture-plan; blocker-decisions | 4.2; D-0 |
| R2.4 | no single status model | Proposed, Provisional, Owner-Approved, Implemented, Frozen, Rejected, Superseded with assigner/evidence/allowed/restrictions; decision registry | production-architecture-plan | 3 |
| R2.5 | ACR-004 monolithic | parts A-E with owner, contracts, dependencies, stage, acceptance | blocker-decisions; production-architecture-plan | 2; 7 |
| R2.5b | PS-1 must close only Core prerequisites (v1 mixed more) | PS-1 contains CORE-07/08/09 (part A); B, C, E are later tasks | module-boundaries | C |
| R2.6 | no minimal Core model | sections 5.1-5.8 | production-architecture-plan | 5 |
| R2.7 | CORE-11 absent from the v1 roadmap stage list | included in PS-1 (G1), merge-after for PERS-01 | module-boundaries; agent-task-breakdown | C; CORE-11 |
| R2.8 | single mutation path not formal | MB-1, handle classes, enforcement 1-5, TEST-06, XT-02 | production-architecture-plan; integration-and-qa-plan | 6; 7 |
| R2.9 | persistence requirements incomplete; spike vs production | requirements list, spike separation, FaultFs, directory fsync, versioned formats | production-architecture-plan; integration-and-qa-plan | 8; 8 |
| R2.10 | WorldGen concerns mixed | six concerns, pure generator parallel on read-only API | production-architecture-plan | 9 |
| R2.11 | networking/client mixed | seven concerns and order; embedded and dedicated share SRV-01 | production-architecture-plan | 10 |
| R2.12.1 | emulator smoke blocks Core | removed; CI-host only | implementation-roadmap | 2 |
| R2.12.2 | device smoke blocks Processing | removed; synthetic budget tests | implementation-roadmap | 2, PS-2 |
| R2.12.3 | renderer validation inside Backends stage | DG-1 separate; G5 independent | implementation-roadmap | 2, 4 |
| R2.12.4 | backend conformance coupled with renderer | TEST-03a generic suite in `cube-integration-tests` | integration-and-qa-plan | 6 |
| R2.12.5 | CI spec vs running CI | GOV-05 vs GOV-06 with separate evidence | integration-and-qa-plan | 3 |
| R2.12.6-9 | concurrency, fault injection, negative mutation, property tests | table of kinds with required-from stage | integration-and-qa-plan | 6 |
| R2.12.10 | golden hashes vs requirements | CI-host equality in PS-11, cross-device in DG-2 | integration-and-qa-plan; roadmap | 6; 4 |
| R3 | ownership matrix | six-column matrix; unassigned modules as owner decisions | module-boundaries | A.2 |
| S1 | v1 PS-1 acceptance required an Android emulator run | removed | implementation-roadmap | 2 |
| S2 | v1 roadmap task list of the Core stage omitted CORE-11 and TEST-02 | generated task lists from one data file | implementation-roadmap | 3 |
| S3 | v1 API-04 had no base-provider port, so recovery would need WorldGen | `IBaseProvider` in API-04 | module-boundaries | B, B.1 |
| S4 | v1 gave Agent I files inside other crates (several owners per crate) | `cube-integration-tests` leaf crate, rule R-DEVCYCLE; validator enforces one owner per crate | module-boundaries | A.1 |
| S5 | v1 WorldGen grid row had an L2 dependency it did not need | removed | module-boundaries | B |
| S6 | v1 assigned providers to backend owners although O-1 is open | marked "pending O-1" | module-boundaries; task tables | A.2 |
| S7 | v1 HIST-04 referenced server design from PS-7 | split: HIST-04 (scope abstraction) and SRV-07 | agent-task-breakdown | HIST-04, SRV-07 |
| S8 | WorldGen/server/client/adapter tasks carried invented owners | `UNASSIGNED` with rule R-UNOWNED | agent-task-breakdown | headers |
| S9 | roadmap-v1 gates 6, 8, 9 not covered | stated as outside this plan | production-architecture-plan | 2.2, 12 |
| S10 | v1 deferred Core data (C-05, C-10…C-15) was inside the Core-stage tasks | moved to consumer stages as separate Core tasks | module-boundaries | C |

## 4. Status of findings

**Resolved by this revision (subject to owner acceptance):** R2.1, R2.2 (all sub-items), R2.3, R2.4, R2.5 (structure), R2.6 (as a proposal), R2.7, R2.8 (design and tests specified, nothing implemented), R2.9, R2.10, R2.11, R2.12, R3 (matrix; owners remain decisions), S1-S10.
**Open (cannot be closed by documents):**
1. The Geometry Processing Specification (RG-2) is absent: **G2 cannot close**, so every production merge from PS-4 onward is blocked. Options for the owner: (a) supply it; (b) record an owner alternative for PR-01…PR-08 (AOD-24 forbids a general assumption); (c) accept Provisional PR contracts with a revalidation gate and relax the G2 requirement for specific modules (not recommended; weakens the invariant).
2. The Voxel Architecture (RG-1) is absent: PC-1 stays Provisional; C-10…C-12 and the persistence format cannot be Owner-Approved.
3. Owners for WorldGen, Networking Adapter, Server Runtime, Client Runtime, Engine Adapter, Import/Export, Derived Data Providers (O-1), Modifier Evaluation (O-2).
4. Confirmation of AOD-01…24 in the repository (currently agent-transcribed).
5. ACR-001/002/003/004 undecided; D-K, D-T, D-C, D-R1, D-R2, D-R6, D-R8, OD-R4, OD-R5, OAM-09, OAM-10.
6. GDD/TDD and original AU-1.0 absent; scope of gameplay, content and MMO beyond this plan.
7. Numeric performance thresholds (device data needed).

## 5. Decisions requiring your confirmation

1. The canonical stage names and mapping (and the pointer lines in the QA Pack and roadmap-v1 if adopted).
2. The ADG as the single dependency authority, with the S/I semantics.
3. The status model, the decision registry and the rule that chat is not a confirmation source.
4. Full P0, P0-alpha (C-01, C-02, C-03, C-04, C-06, C-16, C-17, C-18, C-19) and Provisional PC-1 (C-07, C-08, C-09) as defined; **a change to the QA-Pack rule "P0 approved gates stage 1" (quoted)** (PS-1 gate = P0-alpha).
5. ACR-004 split A-E and the rule that PS-1 closes only A.
6. The minimal Core model (section 5 of the architecture plan), incl. `CommitBatch` multi-layer atomicity and closed `LayerData`.
7. The mutation-boundary design (capability handles).
8. Processing gate G2 as a prerequisite for production integration (stricter than the QA Pack).
9. The new crate `cube-integration-tests`, the new port `IBaseProvider`, `WorldSeed` and `GeneratorVersion` value types in Core.
10. Owner assignments (GOV-03) and the choices listed in `architecture-blocker-decisions.md`.

## 6. Provisional decisions in this revision

PC-1 (C-07, C-08, C-09 with baseline BL-1); every item marked "Proposed" in the architecture plan section 5 until its D-xx decision is made; the closed-enum representation set; edge 16 and `i8` level; quantised density; integer transforms; R-A revision model; C1 layer model; thresholds. All remain Proposed until a registry entry exists.

## 7. Changes that touch previously agreed architecture

| Change | Relation to prior agreements | Needs owner approval |
|---|---|---|
| PS-1 gate is P0-alpha, not full P0 | QA Pack stage 0/1: "P0 approved gates stage 1" | yes |
| C-05 and C-10…C-15 moved out of PS-1 | QA Pack stage 1 validation list includes bounds, transform, material mapping | yes |
| G2 required for production merges of Editing and later | QA Pack: Orchestration may start once the API stage is frozen | yes |
| `ActorKind` on `OperationId`; handles; system-commit port in Core | AOD-20 (recorded), ACR-004 A | via ACR-004 |
| `IBaseProvider` port; Persistence recovery via port | AOD-21 load order unchanged (base, snapshot, journal tail); mechanism new | yes |
| `WorldSeed`, `GeneratorVersion` as Core value types | AOD-09 (WorldGen), AOD-21 | yes |
| New PS-11 stage; PS-12 before PS-13 | QA Pack numbering differs | yes |
| `cube-integration-tests` crate and R-DEVCYCLE | new | yes |
| Gameplay, content and MMO gates not covered | roadmap-v1 gates 6, 8, 9 | scope decision |

## 8. Readiness assessment

**PS-0 (Governance and Architecture Freeze): documents ready for review; the stage is NOT complete.** Exit conditions G0 and their state: GOV-01 (decisions) not done; GOV-02 (P0-alpha approval, PC-1 register) not done; GOV-03 (owners) not done; GOV-04 (docs-only baseline) not done; GOV-05 (CI specification) drafted in `integration-and-qa-plan.md` section 3, not approved; GOV-06 (CI running) not done; TEST-01 and XT-01 not done. 0 of 8 gate tasks are complete. Preparing documents does not complete the stage.
**PS-1 (Geometry Core Foundation): NOT ready to start.** Unmet entry criteria (architecture plan section 14): owner-confirmed decisions (ACR-001, ACR-003, ACR-004 A and D, D-R1, D-R2, D-R6, D-R8, D-K, D-T, D-C, D-0); P0-alpha Owner-Approved; named Core Agent, Integration Owner and Agent I; docs-only baseline; running CI. Not required for PS-1: devices, emulator, the Geometry Processing Specification, WorldGen/server/client owners.

## 9. Recommendations

1. Decide GOV-01 first (single owner session): the 12 listed decisions are mostly accept/amend of proposals already written.
2. Countersign AOD-01…24 in the repository (registry) so they stop being agent transcriptions.
3. Run GOV-04 (baseline) and GOV-06 (CI) in parallel with the decisions.
4. Supply the Geometry Processing Specification early or decide the alternative: it is the longest external dependency of the whole plan.
5. Start SPK-PERS and SPK-NET and the device acquisition (DEV-01) now; none of them blocks PS-1.
6. Order PS-1 work: CORE-01/02/07 first, then CORE-03/05, CORE-06, CORE-08/09, CORE-10 (freeze), then CORE-11 and the QA suites.
7. Run a second independent audit of this set (the ADG validation is mechanical; semantic correctness still needs review).

## 10. Re-audit of the seven documents

| # | Check | Result | Detail |
|---|---|---|---|
| 1 | no unfilled placeholders | PASS |  |
| 2 | all task ids mentioned in the seven documents exist in the ADG | PASS |  |
| 3 | no unqualified "Stage n"/"Gate n" reference | PASS |  |
| 4 | PS-n (0..14), Gn (0..14), DG-n (1..3), RG-n (1..2) references are valid | PASS | [] |
| 5 | P0-alpha contract list identical wherever it is spelled out | PASS | occurrences {'production-architecture-plan.md': 2, 'module-boundaries.md': 0, 'implementation-roadmap.md': 1, 'agent-task-breakdown.md': 0, 'architecture-blocker-decisions.md': 1, 'production-plan-revision-v2-report.md': 1}; deviating [] |
| 6 | PC-1 is C-07, C-08, C-09 wherever it is spelled out | PASS | [('architecture-blocker-decisions.md', 'C-07, C-08, C-09')] |
| 7 | no bare "Approved" claim without Owner- prefix or negation (manual list) | PASS |  |
| 8 | architecture-plan sections referenced elsewhere exist (3.3, 5, 6, 8, 12, 14, 15, 4, 2) | PASS | [] |
| 9 | every ADG task has a row in the task breakdown | PASS | [] |
| 10 | every task id appears in the roadmap stage blocks | PASS | [] |
| 11 | UNASSIGNED owners appear only in PS-11+ (and the transport spike) | PASS | [] |
| 12 | roadmap and task breakdown do not restate dependency lists | PASS |  |
| 13 | ADG parts A-E present | PASS |  |
| 14 | legacy status word "Draft" not used as a v2 status | PASS |  |

### 10.1 Remaining discrepancies and weaknesses (not closable by documents)

1. **The Processing gate also gates G10 indirectly:** G10 contains the integration test ITG-01, whose merge-after edge is G4, hence G2 and the Geometry Processing Specification. The persistence modules themselves (PERS-01…PERS-09) do not wait for G2. If you prefer persistence to close independently of Processing, move ITG-01 to a later gate (owner decision).
2. **SPK-NET and WGEN-01 have UNASSIGNED owners but start early** (PS-0 and right after the Core API freeze). R-UNOWNED therefore needs GOV-03 to name the networking and WorldGen owners (or the Integration Owner to run the spike) before those tasks start; until then they cannot start.
3. **Provider tasks (VOX-02, VOX-03, VOX-05, SDF-02, SDF-03, MSH-03)** carry the backend agents as owners but are marked "pending O-1"; assigning derived-data providers is an owner decision that this revision does not make.
4. **The QA Pack and roadmap-v1 are not edited**; they keep their old numbering until the owner adopts the mapping.
5. **Task acceptance lines are one-liners**; each owner must expand them into a checklist at stage start (tests and gate conditions are fixed, the detailed cases are not).
6. **archcheck rules for the mutation boundary cannot be exercised until the crates exist**; XT-02 is specified, not implemented.
7. **Numeric thresholds** (latency, memory, benchmark limits) are not set anywhere; they depend on device data.
8. **The two external reviews were not available** (section 1); findings in them that are not among sections 3-4 are not covered.
9. **Residual "approved" wording** refers to accepted change requests (a process term), not to contract approval; contract approval always reads Owner-Approved.
