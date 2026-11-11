# Delivery plan: from the current state to M2 (and the shape of M3)

**Status:** **Proposed** / **Pending Owner Approval**. It authorises nothing. **No dates and no effort figures are given: none exists in the repository** (the missing inputs are listed in section 6). Task ids are those of [`M2_TASK_GRAPH.md`](M2_TASK_GRAPH.md); contracts TC-nn of [`M2_MINIMUM_CONTRACTS.md`](../../api-contracts/M2_MINIMUM_CONTRACTS.md); decisions ODM-nn of [`OWNER_DECISIONS_REQUIRED.md`](OWNER_DECISIONS_REQUIRED.md); GOV-nn of the Production Plan.

## 1. Where we start (verified)

PS-0 is open (0 of 8 gate tasks), PS-1 is not started, milestone M0 is in progress. No production code, no CI, `master` unprotected, 38 commits of mixed spike code and documentation on a branch that is not mergeable as is. Nothing in this plan changes that status.

## 2. Phase 0: Pre-implementation

| Step | Tasks / work | Depends on | Owner | Output | Completion criterion | Blocking decisions | Parallel? |
|---|---|---|---|---|---|---|---|
| 0.1 Decide the first batch | GOV-01 items 14 and 15 first (D-E parameters; branch option), then the Core items; ODM-01, ODM-10 | none | Project Owner | registry entries with a confirmation source | entries exist and are confirmed by the agreed mechanism | ODM-10 (how decisions are confirmed) | sequential at its start; the rest of GOV-01 can follow in batches |
| 0.2 Integrate documentation | PRs D1...D4 ([`REPOSITORY_AND_MERGE_REVIEW.md`](REPOSITORY_AND_MERGE_REVIEW.md) section 4C) | 0.1 (ODM-01), fixes of two scripts | Integration Owner prepares, Project Owner approves | documentation on `master` as Proposed/records | PRs merged; `master` has no spike code | ODM-01 | D3 and D4 parallel after D1 and D2 |
| 0.3 Protect `master` with the documentation check | workflow `docs-validate`, branch protection | 0.2 for the scripts, ODM-09 | Agent I (workflow), Project Owner (settings) | a required status check on `master` | the check ran red on a seeded failure and green on `master`; API shows `protected: true` | ODM-09 | parallel with 0.4 |
| 0.4 Close PS-0 | GOV-02 (P0-alpha, PC-1, status model), GOV-03 (owners or deferral), GOV-04 (baseline), GOV-05 (CI specification), then GOV-06 (CI skeleton), TEST-01, XT-01 | 0.1; GOV-06 after GOV-04 and GOV-05 | Project Owner, Integration Owner, Agent I | registry entries, baseline, running CI with required checks, archcheck v0, testkit | gate G0: all 8 tasks merged and accepted with the evidence listed in [`ps-0-exit-checklist.md`](../../production/ps-0-exit-checklist.md) | the 15 GOV-01 decisions, X-1...X-4 | GOV-02 and GOV-03 parallel to GOV-04/05; GOV-06 then TEST-01 and XT-01 parallel |
| 0.5 Register temporary contracts | TC-01...TC-10 as Provisional entries | 0.1 | Project Owner | Provisional register in the registry | each TC has an expiry and an owner entry | ODM-02, ODM-03, ODM-07, ODM-11 | parallel with 0.4 |
| 0.6 Accept the M2 scope and graph | M2-T/M2 split, acceptance criteria, ADG change proposals (M2 slice, Game Rules module, D-01/D-02 rule, relaxations) | none | Project Owner, Integration Owner | accepted scope; ADG change PR | scope recorded; ADG changes merged with regenerated tables | ODM-06, ODM-14, ODM-05 | parallel |
| 0.7 Owners | owners or written deferral: WorldGen, Networking, Server, Client, Engine Adapter, Gameplay, Content, Art/Audio | none | Project Owner | ownership table | each unassigned module in the M2 graph has an owner or a dated deferral | ODM-04 | parallel; **needed per wave, not all at once** (section 3) |
| 0.8 Design and experiments (L1/L2) | GM-01, GM-02, GM-03, GM-04, TGAP-05, SPK-PERS, GM-16; the fixes of the audit scripts | experiments: owner go-ahead (ODM-12) | Gameplay, Content, Art owners (when named), Integration Owner, Agent H | design documents, scenarios, content tables, findings | each document reviewed; each experiment ends with a written finding | ODM-12 | fully parallel |

## 3. Phase M2-T: Technical Playable

Minimum conditions to **start** (the answer to "what is the smallest safe start"):

| # | Condition | For which wave |
|---|---|---|
| S-1 | Gate G0 satisfied: baseline, CI skeleton with required checks, archcheck v0, testkit, registry | all |
| S-2 | The Core decisions of GOV-01: items 1, 4-9, 11, 13 and the coordinate decisions (items 10, 12) recorded; `VoxelPos` width, level semantics and world bound decided (TC-03) | W1 |
| S-3 | TC-10 (threading) accepted | W2 |
| S-4 | TC-01 accepted (or the specification supplied) | W2 |
| S-5 | Owners for W1 work: **existing roles only** (Core Agent, Agent I, Integration Owner) | W1 |
| S-6 | Owners for WorldGen, Networking, Server, Client, Engine Adapter named (or the work cannot start) | W4, W5 |
| S-7 | Owner for Game Rules named, module accepted, entity-model spike done | GM tasks in W3/W4 |
| S-8 | TC-02, TC-08 registered; vertical extent decided | W4 |

Consequence: **the first waves need no unassigned owner**. The Technology Track can start with W1 (Core) as soon as G0 and the Core decisions exist, while the owners of the later waves are still being named.

| Wave | Tasks (milestone M2-T) | Depends on | Owner | Output | Completion criterion | Parallel inside the wave |
|---|---|---|---|---|---|---|
| W1 Core | CORE-01...CORE-11, CORE-16, TEST-02, TEST-05, TEST-06, XT-02, INT-01 | G0, S-2 | Core Agent, Agent I, Integration Owner | `cube-core`, mutation boundary, API snapshot, golden hashes | G1 members merged; contract tests, property and loom tests, compile-fail tests green; `INT-01` recorded | CORE-01/02/07 start together; TEST-02/05/06 follow their targets |
| W2 Processing and API | PROC-01...PROC-05, TEST-08; API-01...API-05 | CORE-10 (API freeze) | Processing Agent, Integration Owner, Agent I | snapshots, executor, cancellation, stale rejection; editing types and ports | TC-01/TC-10 conformance; concurrency suites green; API snapshot test | Processing and API-01/03/04 in parallel; API-02 after PROC-01/04 |
| W3 Edit path, meshing, queries | ORCH-01...ORCH-05, TEST-03a, TEST-10, VOX-01, VOX-02, VOX-04, VOX-05, TGAP-01 | W2 | Agent A, Agent B, Agent I, query owner | user commit path; voxel backend; faces provider; collision and ray queries | edits only as user commits; backend conformance; query correctness on chunk borders and negative coordinates | ORCH and VOX in parallel |
| W4 World and storage | WGEN-01...WGEN-04, TEST-09; PERS-01, PERS-02, PERS-03, PERS-08 (after SPK-PERS); GM-10...GM-13 | W3 (WGEN-03 needs the system-commit port from W1) | WorldGen owner, Agent H, Gameplay owner, Agent I | generator, service, materialiser; snapshot-only persistence v0; Game Rules skeleton, player, controller, interaction | determinism suite; world saved and reloaded in tests; controller replay tests | WorldGen, Persistence and Game Rules in parallel |
| W5 Runtime and client | NET-01...NET-03, SRV-01, SRV-03, SRV-04, TGAP-03a, CLI-00, CLI-01, CLI-02, CLI-04, CLI-05, ADP-01...ADP-03, TGAP-04, GM-14, GM-15, GM-16 | W3, W4 | Networking, Server, Client, Engine Adapter owners; Agent I | embedded host over the in-process transport; client; adapter; menus and camera; reproducible build | adapter audit (no authority in scripts); build from a clean checkout | server/net and client/adapter in parallel after NET-01 |
| W6 Integration | XI-01, TGAP-06, XI-02 | W5 | Integration Owner, Agent I | headless scenario, stability harness, M2-T build and evidence | T-01...T-13 demonstrated | XI-01 then TGAP-06 and XI-02 |

**Critical path (tasks, not time):** `GOV-01 -> GOV-04 -> GOV-06 -> TEST-01 -> CORE-01 -> CORE-03 -> CORE-06 -> CORE-10 -> PROC-01 -> PROC-05 -> TEST-08 -> API-02 -> NET-01 -> NET-03 -> SRV-01 -> SRV-03 -> GM-12 -> XI-02` (18 tasks). The chain runs through governance, Core, Processing, the Editing API and then the embedded runtime; persistence and world generation are off the critical path but gate `XI-01`.

## 4. Phase M2: First Playable

| Wave | Tasks | Depends on | Owner | Output | Completion criterion | Notes |
|---|---|---|---|---|---|---|
| M2-A Durable persistence | PERS-04, PERS-05, PERS-06, ITG-01, ITG-02, TEST-04, QA-P2, TGAP-02 | M2-T W4/W5; `GOV-11` | Agent H, Agent I | journal, compaction, recovery, record store; crash matrix subset | TC-08 retired; crash after every compaction step recovers a verified state | independent of Game Rules |
| M2-B Game Rules | GM-20...GM-29, GM-30, GM-33, TGAP-03b | W4 Game Rules skeleton; TGAP-08 for creatures | Gameplay owner, Client Runtime owner, Networking owner | items, inventory, equipment, health, combat, creatures and AI, loot, respawn, player record, replication | scenarios FPS-05...FPS-08 pass headless | most tasks parallel after GM-20/23/25 |
| M2-C World content | TGAP-07 (second variant), TGAP-08 (spawn step) | WGEN-01, WGEN-03 | WorldGen owner | generator version 2 | determinism and border tests | parallel with M2-B |
| M2-D Presentation | GM-31, GM-32 | GM-33 | Engine Adapter owner, Art owner | HUD, inventory UI, placeholder creature models | FPS-07/08 visible | |
| M2-E Integration | XI-03, XI-04 | all | Integration Owner | M2 build, playtest note, evidence package | FPS-01...FPS-17 demonstrated; outside tester session; owner records acceptance | acceptance is the owner's |

Critical path to M2 (23 tasks): the M2-T path up to `SRV-03`, then `GM-12 -> GM-24 -> GM-30 -> GM-33 -> GM-31 -> XI-03 -> XI-04`.

## 5. Phase M3: shape only

No M3 task graph is produced: velocity, content cost and several decisions (Minimum or Recommended slice, mobile in M3, co-op spike) are unknown ([`M3_SCOPE_REVIEW.md`](M3_SCOPE_REVIEW.md)). Inputs: M2 accepted; owners for content and art; TC-01, TC-02, TC-08 expired or renewed; MAC-1 done if mobile is included; a plan for the Game Rules additions (abilities, crafting, quests, boss) and for the generator additions.

## 6. What is unknown

| Unknown | Needed to resolve it |
|---|---|
| Effort and duration of every task | task estimates by the assigned owners; velocity measured on W1 and W2; benchmarks |
| Staffing: how many parallel workers exist | the owner (ODM-04) |
| Whether Provisional contracts satisfy AOD-24's condition | the owner (ODM-02) |
| When the Processing Specification and Voxel Architecture will exist | the owner |
| Device availability | the owner (`DEV-01`) |
| Cost of RegionStamp, meshing and streaming on target hardware | benchmarks B-1...B-10 |

## 7. Parallel and sequential work

**Can start now without waiting for the architecture work to finish** (no production code involved): documentation integration prep and the script fixes; decisions; GM-01, GM-02, GM-03, GM-04, TGAP-05; GM-16; the owner naming; the experiments after go-ahead (SPK-PERS, entity-model spike, controller-feel prototype).

**Parallel once G0 and the Core decisions exist:** W1 alone on the Technology Track; Game Rules design and stand-ins continue in parallel.

**Strictly sequential:** `GOV-01` before `GOV-04`/`GOV-05`; `GOV-04` and `GOV-05` before `GOV-06`; `GOV-06` before any production PR; `CORE-10` (API freeze) before Processing, API and Game Rules code; Processing (`PROC-01`/`PROC-04`) before `API-02`; the Editing API before orchestration; the embedded runtime (`SRV-01`/`SRV-03`) before the controller can be run for real.

## 8. Risks specific to the plan

| Risk | Indicator | Mitigation |
|---|---|---|
| The path waits for decisions, not code | W1 cannot start for lack of the Core decisions | decide GOV-01 items 14, 15 first, then the Core batch; answer in batches |
| Owners are missing for W4/W5 | S-6 not met when W3 ends | name owners during W1-W3; the first three waves do not need them |
| Provisional contracts are rejected | ODM-02 answer is "wait" | the path through `API-02` is blocked by a document; escalate the document |
| Game Rules scope grows | stretch items (XP, dodge, audio) enter | acceptance list is fixed (FPS-01...FPS-17) |
| Temporary contracts persist | TC expiry passes unnoticed | register plus CI check on expiry ([`M2_MINIMUM_CONTRACTS.md`](../../api-contracts/M2_MINIMUM_CONTRACTS.md) section 5) |
