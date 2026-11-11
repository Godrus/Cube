# Cube Production Plan v2.1: Revision Report (final consistency review)

**Status:** report on a **Proposed** revision. Nothing in the documents is approved; no contract, ACR, AOD, OD, OAM or D-decision changed status; no code, merge, commit, push or branch change was made; no implementation started. Work was done outside the repository and then written into the working tree of `docs/production/` **without committing**.
**Set (v2.1):** [`production-architecture-plan.md`](../../production/production-architecture-plan.md), [`module-boundaries.md`](../../production/module-boundaries.md) (with the ADG), [`implementation-roadmap.md`](../../production/implementation-roadmap.md), [`agent-task-breakdown.md`](../../production/agent-task-breakdown.md), [`architecture-blocker-decisions.md`](../../production/architecture-blocker-decisions.md), [`integration-and-qa-plan.md`](../../production/integration-and-qa-plan.md), this report, and the new [`ps-0-exit-checklist.md`](../../production/ps-0-exit-checklist.md). The earlier [`production-plan-revision-v2-report.md`](production-plan-revision-v2-report.md) stays as a historical record with a "superseded" banner.

## 1. Basis, method and limits

* Reviewed: the seven v2 documents as committed in `22c70c7`; the audit report of 2026-11-05; the AOD, ACR, OD, OAM and D decision records; the requirements of the review request (eight mandatory checks).
* Method: (a) a manual read for contradictions between documents; (b) every dependency, owner and stage claim re-derived from the single data file and machine-validated (`_build/validate.py`; 34 structural checks in section 7); (c) a document-level re-audit (`_build/reaudit.py`, section 10); (d) all changes carry a trace tag V-nn (section 3, trace table 7.2).
* **Fact:** the texts of the earlier independent audits and of the "second review" were not available to me. Findings V-01…V-19 are the ones I found by checking the eight mandatory points; findings outside them are not covered.
* **Fact:** no CI exists in the repository today (no workflow files in any branch). CI and archcheck are therefore **specified and made gate tasks with evidence requirements**, not observed.

## 2. Verdict

**READY FOR OWNER APPROVAL** (of this document set).
Meaning, precisely: the seven documents are internally consistent after the corrections below; the dependency graph is complete and acyclic; the open items are owner decisions or external inputs and are listed, not hidden. It does **not** mean that PS-0 or PS-1 is complete or may start: **PS-0 is NOT complete (0 of 8 gate tasks done, 0 of 15 decisions confirmed) and PS-1 is NOT ready to start** ([`ps-0-exit-checklist.md`](../../production/ps-0-exit-checklist.md)).
Two conditions would change the verdict to NOT READY: (1) the owner rejects the canonical numbering or the ADG semantics (then the documents must be regenerated), (2) a check in section 10 fails after any later edit.

## 3. Contradictions found and corrected

| Id | Contradiction (evidence) | Correction | Documents | Check point |
|---|---|---|---|---|
| V-01 | CORE-10 (freeze of CORE-01…07) did not depend on CORE-04 (Transform) although its scope names it | CORE-10 depends on CORE-04 | module-boundaries C; agent-task-breakdown | 1 |
| V-02 | the architecture plan said ACR-004 part A closes with TEST-06 in PS-1, but TEST-06 was a member of G4 (PS-4) | TEST-06 is the PS-1 mutation-boundary test (member of G1); the Orchestration-path tests are the new TEST-10 (G4); LEG-02 merges after TEST-10 | all six | 1, 2 |
| V-03 | CORE-07 (PS-1) contained WorldSeed and GeneratorVersion, support types of WorldGen and Persistence: a hidden PS-1 coupling to ACR-004 parts B and C | moved to the new CORE-16 (PS-2); WGEN-01 and PERS-08 depend on it; PS-1 members are owned only by Core Agent, Agent I, Integration Owner | module-boundaries; plan 9; roadmap | 2 |
| V-04 | the PS-1 exit said "owner records Implemented", but the status model assigns Implemented to the Integration Owner; no task existed | task INT-01 (member of G1) records Implemented for P0-alpha | plan 3.1, 15; roadmap | 5 |
| V-05 | G2 gated Orchestration (PS-4) but not the Editing API ports (PS-3); the review request requires the Processing exit before production integration of the Editing API | API-02 merges after G2; G3 therefore depends on G2; the computed critical chain now runs through G2 -> API-02 -> G3 | module-boundaries C, D; plan 12; roadmap | 3 |
| V-06 | the rule "no production merge on a Proposed contract" existed only as text; deferred and later contracts had no approval task | owner-approval tasks GOV-16 (C-05), GOV-17 (C-10), GOV-18 (C-11/12), GOV-19 (C-13…15), GOV-20 (tool contracts), GOV-21 (WG), GOV-22 (SR and protocol), GOV-23 (composite and resample contracts), each a merge-after edge of its consumers | module-boundaries C; agent-task-breakdown | 5, 6 |
| V-07 | OD-R4, OD-R5 and D-E had no decision task | GOV-24, GOV-25; D-E and the branch option added to GOV-01 | module-boundaries; blocker-decisions | 6, 7 |
| V-08 | persistence journal appended "through the observer" in one table and through ports in the persistence plan; ORCH-04 lacked API-04; the module grid had no orchestration -> persistence port edge | journal appended through `IOperationJournal` (API-04); ORCH-04 starts after API-04; grid edge "A (journal port only)"; `ITransactionObserver` implemented by History only | module-boundaries A, B, B.1, C | 6 |
| V-09 | the dedicated host and the external transport had no task; the transport suite did not wait for the external transport | SRV-08 (dedicated host over NET-04); TEST-03b waits for NET-04; SRV-01 is the embedded host over the shared runtime | module-boundaries; agent-task-breakdown | 6, 7 |
| V-10 | "Implemented-Provisional" was not a status of the model; the Provisional -> Frozen guard was only text | two-axis status model (approval axis incl. Provisional; maturity axis Implemented -> Frozen only after Owner-Approved); GOV-08 (RG-1 record) and GOV-15 (freeze decision) tasks; checks that GOV-15 depends on GOV-08 and EXT-VA-RG1 | plan 3.1, 4; roadmap; blocker-decisions | 5 |
| V-11 | Part D (start, merge, closing conditions) was hand-written and could diverge from the task edges | Part D is generated from the edges; only extra closing text is hand-written | module-boundaries D | 6 |
| V-12 | GOV-04 (baseline option) and GOV-05 (CI specification with licence allow-list) did not depend on the owner decisions they need | GOV-04 and GOV-05 depend on GOV-01 | module-boundaries C | 6 |
| V-13 | CI and archcheck were described but nothing required them to be enforceable | GOV-06, XT-01, XT-02 acceptance: required status checks (branch-protection evidence) and red-on-seeded-failure evidence | integration-and-qa-plan 3; plan 14; checklist | 8 |
| V-14 | broken cross-reference ("section 2.3 of the roadmap") | corrected to [`implementation-roadmap.md`](../../production/implementation-roadmap.md) section 4 | plan 1 | 6 |
| V-15 | "at most four crates exist when PS-1 starts" contradicted the crate creation stages | "during PS-1" | module-boundaries A.1 | 6 |
| V-16 | PS-1 Core model silently assumed the single-layer model (OAM-09 is decided later, in GOV-11) | explicit compatibility note: C1 baseline, C2 additive, PS-1 does not wait for OAM-09 | plan 5 | 7 |
| V-17 | the contracts closing ACR-004 part A (SC-xx drafts) had no approval scope: P0-alpha lists only C-xx contracts | P0-alpha explicitly includes the part A amendments to C-01/C-16/C-17 (texts SC-01, SC-02, SC-05, SC-10); no other SC/WG/WP/SR contract | plan 4.1 | 2, 5 |
| V-18 | the integrity checks covered 16 properties; points 1, 2, 5 and 8 of the review request had no automated check | additional checks (section 7), including G1 isolation from WorldGen, Persistence, History, Processing, devices | build/validate | 1-8 |
| V-19 | the v2 report described v2 counts and statements that no longer hold | banner "superseded by v2.1" | production-plan-revision-v2-report.md | n/a |

## 4. The eight mandatory checks: result

| # | Check | Result |
|---|---|---|
| 1 | CORE-01…CORE-11 consistent between roadmap, task breakdown, module-boundaries, QA plan | PASS after V-01, V-02, V-03; the PS-1 members are generated from one data file; the phrase "CORE-01…CORE-11" equals the G1 Core membership (check in section 10) |
| 2 | ACR-004 A-E; what PS-1 needs; no hidden dependency on WorldGen, Persistence, History | PS-1 contains only part A (and the part D principle as a decision); G1 has no ancestor and no member in WorldGen, Persistence, History, Processing, Editing API, Orchestration, backends, network, client, adapter, device tasks (check) |
| 3 | Critical path through the Processing exit before production integration of the Editing API | PASS after V-05; chain printed in section 7 |
| 4 | Emulator and devices are not acceptance gates for PS-1 | PASS: `EXT-DEV` is an ancestor only of DG-1…DG-3 and G14 (check); PS-1 exit text has no emulator or device condition |
| 5 | P0, P0-alpha, PC-1 and the statuses Owner-Approved, Provisional, Implemented, Frozen | PASS after V-04, V-10, V-17; Frozen of a Provisional contract is impossible by model and by graph (GOV-15 needs GOV-08) |
| 6 | ADG: existence, acyclicity, completeness, one owner per crate, roadmap agreement, start versus merge conditions | PASS (section 7); Part D generated; V-06…V-12 fixed missing dependencies |
| 7 | All ACR, AOD, OD, OAM, D-decisions against the agreed architecture | table in section 5; three deviations need owner approval |
| 8 | CI and archcheck are real required checks | **Specified and gated, not yet real:** no CI exists today; GOV-06, XT-01, XT-02 acceptance requires required-status registration with evidence; PS-0 cannot close without it |

## 5. Conformance with previously agreed decisions

| Decision(s) | Where reflected | Verdict |
|---|---|---|
| AOD-01…04 (Rust, Godot first client, GDExtension, Core pure Rust) | crates; no `godot` outside the adapter; archcheck | consistent |
| AOD-05, AOD-13, AOD-14, AOD-22 (headless server, authority, client not truth, embedded and dedicated share one runtime, in-process vs external transport) | SRV-01 (embedded), SRV-08 (dedicated), TEST-03b (both transports), TEST-07 | consistent |
| AOD-06, AOD-07, AOD-12 (replaceable frontend, GDScript glue, no child processes) | ADP-02, ADP-03, archcheck | consistent |
| AOD-08, AOD-11 (mobile mandatory, emulator is not a device) | DG-1…DG-3 and PS-14; emulator never an acceptance gate | consistent |
| AOD-09, AOD-16, AOD-20 (pure deterministic WorldGen, separate system boundary, hybrid C3) | WGEN-01 (no floats), WGEN-03 (Materialiser via SystemCommit), ACR-004 A-E | consistent |
| AOD-10 (faces reference, greedy experimental) | DG-1, GOV-13; G5 independent of the renderer | consistent |
| AOD-15 (user changes only through the Editing API) | `UserCommit` only in Orchestration; LEG-01, LEG-02 | consistent |
| AOD-17, AOD-21 (seed + version, journal, snapshots, protocol, load order, no format approved) | PERS-02…PERS-06, GOV-10; load order via `IBaseProvider` | consistent; new port needs approval |
| AOD-18, AOD-19 (P0/P1 Draft, ACR not rewritten automatically) | all statuses Proposed; ACR-004 split but Pending | consistent |
| AOD-23, AOD-24 (licence policy, conditional sources) | D-E allow-list; PC-1 on BL-1; the Geometry Processing Specification never replaced by a general assumption (G2 needs GOV-07) | consistent |
| ACR-001/002/003/004 | GOV-01, GOV-11 | all Pending |
| OD-R1, R2, R6, R8 | GOV-01 | Proposed |
| OD-R3, OD-R7 | not blocking; deferred (listed in the QA plan) | not tasked; intentionally deferred |
| OD-R4, OD-R5 | GOV-24, GOV-25 | Proposed |
| OAM-01, 02, 03, 07, 08 | recorded earlier as AOD-20…24 | consistent |
| OAM-04, 05, 06 | DG-1/GOV-13, SPK-NET/GOV-12, DG gates | consistent |
| OAM-09, OAM-10 | GOV-11, GOV-01 | Proposed |
| D-0, D-K, D-T, D-C, D-R8, D-E, D-P, D-NET, D-OWN, D-REG | GOV-01, GOV-02, GOV-03, GOV-10, GOV-12 | Proposed |

**Deviations from earlier documents (need owner approval, restated from the v2 report):** (1) PS-1 gate is P0-alpha instead of full P0 (QA Pack stage-0 rule), (2) C-05 and C-10…C-15 are moved out of PS-1 (QA Pack stage-1 list), (3) G2 gates production merges of the Editing API and later (stricter than the QA Pack). Additional new structure needing approval: `cube-integration-tests`, `IBaseProvider`, `CommitBatch`, closed `LayerData`, the capability handles, CORE-16 world identity types, the `PS-n` naming.

## 6. Decisions still required from the owner

1. Countersign AOD-01…24 in the repository (registry); until then they are recorded, not confirmed. 2. ACR-001, ACR-002, ACR-003. 3. ACR-004 part A and the part D principle now; parts B, C, E at GOV-11. 4. P0-alpha list (with the part A amendments), PC-1 as Provisional with its register entry, the status model and the registry. 5. D-R1, D-R2, D-R6, D-R8, D-K, D-T, D-C, D-0, D-E. 6. The branch option for the docs-only baseline. 7. Owners: Core Agent, Integration Owner, Agent I (before PS-1); WorldGen, networking, server, client, engine adapter, providers (O-1), modifier evaluation (O-2), import/export (O-4) at their stages; the networking and WorldGen owners (or the spike runner) before SPK-NET and WGEN-01 start. 8. Supply or replace the Geometry Processing Specification (RG-2; closes nothing without it). 9. Supply the Voxel Architecture (RG-1). 10. Timing: OAM-09, OAM-10 (principle now), OD-R4, OD-R5. 11. Adopt the canonical names and add pointer lines to the QA Pack and roadmap-v1. 12. Scope after this plan: gameplay, content, MMO (roadmap-v1 gates 6, 8, 9). 13. Acquire devices and macOS/Xcode (does not block PS-1…PS-13). 14. Whether ITG-01 stays in G10 (it makes G10 wait for G2 through G4) or moves later.

## 7. ADG integrity

* Tasks: 147; gates: 15; external inputs: 3; tasks outside any gate (spikes and device acquisition): SPK-PERS, SPK-NET, DEV-01.
* Method: `_build/validate.py` loads the single data file and checks ids, owners, dependency existence, stage monotonicity, gate membership, acyclicity (Kahn) on the union of S and I edges plus gate membership edges, and one owner per crate directory.

| Check | Result |
|---|---|
| graph has no unknown dependency, no cycle, no stage inversion, one owner per crate | PASS |
| G2 (Processing production gate) is an ancestor of every ORCH task | PASS |
| G2 is an ancestor of VOX-01, SDF-01, MSH-01 and SRV-01 | PASS |
| EXT-GPS is an ancestor of G2 (specification gates Processing production) | PASS |
| PS-1 gate G1 does not depend on any device, emulator, WorldGen, Persistence, Processing or networking node | PASS |
| G2 does not depend on EXT-DEV | PASS |
| G5 (backend conformance) does not depend on DG-1 or GOV-13 (renderer) | PASS |
| G10 does not depend on History (HIST-*) or WorldGen (WGEN-*) or devices | PASS |
| PERS-06 depends on PERS-04, PERS-05, CORE-09, API-04, GOV-11 (directly or via S/I) | PASS |
| PERS-04 depends on API-04, CORE-07, CORE-05, PERS-01 | PASS |
| WGEN-02 depends on WGEN-01, PROC-03, PROC-04 and not on Persistence | PASS |
| WGEN-03 depends on CORE-09 and GOV-11 | PASS |
| NET-02 (protocol types) does not depend on any SRV task; SRV-02 depends on NET-02 | PASS |
| CORE-11 is in gate G1 and is a dependency of PERS-01 | PASS |
| PERS-09 depends on CORE-14 and does not depend on COMP-* or HIST-* | PASS |
| every unassigned-owner task is outside PS-0..PS-10 except the networking spike SPK-NET (R-UNOWNED never blocks PS-1) | PASS |
| CORE-10 (API freeze) depends on CORE-01..CORE-07 including CORE-04 | PASS |
| G1 members are owned only by Core Agent, Agent I, Integration Owner (no WorldGen, Persistence, History, Processing owner) | PASS |
| G1 has no ancestor in WorldGen, Persistence, History, Processing, Editing API, Orchestration, backends, network, client, adapter or device tasks | PASS |
| G1 contains CORE-11, TEST-02, TEST-05, TEST-06, XT-02, INT-01 and CORE-07..09 | PASS |
| WorldSeed/GeneratorVersion task CORE-16 is outside PS-1 (not in G1) | PASS |
| API-02 (Editing API ports) merges after G2 (Processing production gate) | PASS |
| G3 depends on G2 | PASS |
| no device/emulator input (EXT-DEV) is an ancestor of any gate G0..G13 | PASS |
| GOV-15 (Freeze of PC-1) depends on GOV-08 and EXT-VA-RG1 | PASS |
| CORE-12, CORE-13 depend on the RG-1 revalidation record GOV-08 | PASS |
| every deferred Core contract task has an owner-approval ancestor (CORE-12:GOV-17, CORE-13:GOV-18, CORE-14:GOV-19, CORE-15:GOV-16) | PASS |
| PROC-06 depends on GOV-24 (OD-R4) and HIST-02 on GOV-25 (OD-R5) | PASS |
| tool/composite/resolution/WorldGen/server merges depend on their contract approvals (GOV-20, GOV-23, GOV-21, GOV-22) | PASS |
| ORCH-04 depends on API-04 (journal port); PERS-04 does not depend on API-02 or on G2 | PASS |
| dedicated host SRV-08 depends on NET-04; transport suite TEST-03b depends on NET-03 and NET-04 | PASS |
| CI and archcheck acceptance require registration as required status checks | PASS |
| GOV-04 and GOV-05 depend on the owner decision GOV-01 | PASS |
| every gate member task belongs to the gate stage and no task is in two gates | PASS |

* Longest dependency chain to G14 (21 task nodes): `GOV-01 -> GOV-04 -> GOV-06 -> TEST-01 -> G0 -> CORE-01 -> CORE-03 -> CORE-06 -> CORE-10 -> PROC-01 -> PROC-05 -> TEST-08 -> G2 -> API-02 -> GOV-09 -> G3 -> ORCH-05 -> TEST-10 -> G4 -> ITG-01 -> G10 -> SRV-01 -> SRV-03 -> SRV-04 -> SRV-05 -> G13 -> QA-04 -> G14`

### 7.1 Counts

Tasks, gates, external inputs and tasks outside any gate are in the first line above. New in v2.1: 16 tasks (CORE-16, TEST-10, INT-01, SRV-08, GOV-08 and GOV-15…GOV-25); v2 had 131.

### 7.2 Trace table: every graph change of v2.1

| Finding | Node | Change |
|---|---|---|
| V-01 | CORE-10 | start-after += CORE-04 |
| V-01 | CORE-10 | merge-after += CORE-04 |
| V-03 | CORE-07 | WorldSeed and GeneratorVersion removed from PS-1 (scope and public API) |
| V-03 | CORE-16 | new task (PS-2) |
| V-03 | WGEN-01 | start-after += CORE-16 |
| V-03 | WGEN-01 | merge-after += CORE-16 |
| V-03 | PERS-08 | start-after += CORE-16 |
| V-03 | PERS-08 | merge-after += CORE-16 |
| V-03 | G2 | gate members += CORE-16 |
| V-03 | CORE-09 | scope clarified: callers are outside PS-1 |
| V-02 | G1 | gate members += TEST-06 |
| V-02 | G4 | gate members += TEST-10 |
| V-02 | TEST-06 / TEST-10 | TEST-06 moved to PS-1 (G1); the Orchestration-path tests are TEST-10 (G4); LEG-02 merges after TEST-10 |
| V-04 | G1 | gate members += INT-01 |
| V-05 | API-02 | merge-after += G2 |
| V-06 | CORE-15 | merge-after += GOV-16 |
| V-06 | CORE-12 | merge-after += GOV-17 |
| V-06 | CORE-13 | merge-after += GOV-18 |
| V-06 | CORE-14 | merge-after += GOV-19 |
| V-06 | SCU-02 | merge-after += GOV-20 |
| V-06 | BOO-01 | merge-after += GOV-20 |
| V-06 | COMP-01 | merge-after += GOV-23 |
| V-06 | RES-01 | merge-after += GOV-23 |
| V-06 | WGEN-02 | merge-after += GOV-21 |
| V-06 | WGEN-03 | merge-after += GOV-21 |
| V-06 | NET-02 | merge-after += GOV-22 |
| V-06 | SRV-01 | merge-after += GOV-22 |
| V-06 | G2 | gate members += GOV-16 |
| V-06 | G5 | gate members += GOV-08, GOV-17 |
| V-06 | G6 | gate members += GOV-20 |
| V-06 | G8 | gate members += GOV-19, GOV-23 |
| V-06 | G9 | gate members += GOV-18 |
| V-06 | G11 | gate members += GOV-21 |
| V-06 | G13 | gate members += GOV-22 |
| V-06 | G14 | gate members += GOV-15 |
| V-07 | PROC-06 | merge-after += GOV-24 |
| V-07 | HIST-02 | merge-after += GOV-25 |
| V-07 | G2 | gate members += GOV-24 |
| V-07 | G7 | gate members += GOV-25 |
| V-07 | GOV-01 | scope += D-E and the branch option |
| V-12 | GOV-04 | merge-after += GOV-01 |
| V-12 | GOV-05 | start-after += GOV-01 |
| V-12 | GOV-05 | merge-after += GOV-01 |
| V-08 | ORCH-04 | start-after += API-04 |
| V-09 | TEST-03b | merge-after += NET-04 |
| V-09 | G13 | gate members += SRV-08 |
| V-09 | SRV-01 / SRV-08 | embedded and dedicated hosts are separate tasks over one runtime |
| V-13 | GOV-06 / XT-01 / XT-02 | acceptance requires required-status registration with evidence |

## 8. Remaining weaknesses (not closable by documents)

1. G10 contains ITG-01, whose merge-after edge is G4, hence G2 and the Geometry Processing Specification; the persistence modules themselves do not wait for G2 (decision 14).
2. SPK-NET and WGEN-01 have UNASSIGNED owners but start early; GOV-03 must name them first (rule R-UNOWNED).
3. Provider tasks carry the backend agents provisionally ("pending O-1").
4. The QA Pack and roadmap-v1 are not edited.
5. Task acceptance lines are one-liners; owners expand them at stage start.
6. archcheck rules for the mutation boundary cannot be exercised until the crates exist; XT-02 is specified.
7. Numeric thresholds do not exist.
8. The earlier audit texts were not available (section 1).
9. The checks prove structural consistency, not that the architecture is right; an independent semantic audit remains advisable.

## 9. Recommendations

1. Run GOV-01/GOV-02 as one owner session using [`ps-0-exit-checklist.md`](../../production/ps-0-exit-checklist.md). 2. Start GOV-04 and GOV-06 in parallel. 3. Supply the Geometry Processing Specification early: it is the longest external dependency (it gates API-02 and everything after). 4. Name the Core Agent, Integration Owner and Agent I before anything else. 5. After approval of the document set, regenerate nothing by hand: edit `_build/data.py` and rebuild, so that the ADG stays the single source.

## 10. Re-audit of the documents

| # | Check | Result | Detail |
|---|---|---|---|
| 1 | no unfilled placeholders | PASS |  |
| 2 | all task ids mentioned in the seven documents exist in the ADG | PASS |  |
| 3 | no unqualified "Stage n"/"Gate n" reference | PASS |  |
| 4 | PS-n (0..14), Gn (0..14), DG-n (1..3), RG-n (1..2) references are valid | PASS | [] |
| 5 | P0-alpha contract list identical wherever it is spelled out | PASS | occurrences {'production-architecture-plan.md': 2, 'module-boundaries.md': 0, 'implementation-roadmap.md': 1, 'agent-task-breakdown.md': 0, 'architecture-blocker-decisions.md': 1, 'production-plan-revision-v2.1-report.md': 0, 'production-plan-revision-v2-report.md': 1, 'ps-0-exit-checklist.md': 1};  |
| 6 | PC-1 is C-07, C-08, C-09 wherever it is spelled out | PASS | [('architecture-blocker-decisions.md', 'C-07, C-08, C-09')] |
| 7 | no bare "Approved" claim without Owner- prefix or negation (manual list) | PASS |  |
| 8 | architecture-plan sections referenced elsewhere exist (3.3, 5, 6, 8, 12, 14, 15, 4, 2) | PASS | [] |
| 9 | every ADG task has a row in the task breakdown | PASS | [] |
| 10 | every task id appears in the roadmap stage blocks | PASS | [] |
| 11 | UNASSIGNED owners appear only in PS-11+ (and the transport spike) | PASS | [] |
| 12 | roadmap and task breakdown do not restate dependency lists | PASS |  |
| 13 | ADG parts A-E present | PASS |  |
| 14 | legacy status word "Draft" not used as a v2 status | PASS |  |
| 15 | every "CORE-01…CORE-nn" range in the documents ends at 11 (G1 Core members) or 07 (API freeze scope) | PASS | ranges ['11', '11', '11', '11']; G1 core ['CORE-01', 'CORE-02', 'CORE-03', 'CORE-04', 'CORE-05', 'CORE-06', 'CORE-07', 'CORE-08', 'CORE-09', 'CORE-10', 'CORE-11'] |
| 16 | removed wording absent (Implemented-Provisional, broken section reference) | PASS | [] |
| 17 | no document makes an emulator or device run a condition of PS-1 / G1 | PASS |  |
| 18 | TEST-06 is never described as a PS-4 task | PASS | [] |
| 19 | WorldSeed/GeneratorVersion are never attached to CORE-07 (PS-1) | PASS | [] |
