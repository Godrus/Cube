# M2-T optimised task graph (v2, proposed)

**Status:** **Proposed.** A separate graph. The official ADG (`docs/production/module-boundaries.md`) is **not modified**, no ADG task or gate status changes, and the previous graph (`docs/archive/m2-delivery-review/M2_TASK_GRAPH.md`) is preserved unchanged in the repository as the original. Tables are generator output (`_build/opt_graph.py`, `_build/build_tables.py`, outside the repository); IDs are those of the ADG, and the previous package's `TGAP-`, `GM-`, `XI-`; the one new id is `TGAP-09`. Nothing here authorises work.

## 1. Summary

| Measure | Original v1 (as committed) | B1 (ADG-faithful, acceptance-complete) | **v2 (proposed)** |
|---|---|---|---|
| Tasks in M2-T | 75 | 79 | **74** |
| Pre / M2 tasks | 17 / 27 | 17 / 23 | 17 / **27** |
| Strict critical path (tasks) | 18 | 20 | **18** |
| With rule R-MERGE-G1 | 22 | 24 | **22** (21 without `INT-01`) |
| Dependency edges | | 426 | **350** |
| Levels / maximum width | | 24 / 12 | **22 / 15** |

M2-T task-count changes: 75 -> 74 = `NET-02` and `CLI-05` leave M2-T; `PERS-06` joins (base + snapshot recovery). Against the honest baseline B1 (which contains the journal, compaction and `TEST-04` that the ADG requires for reload): 79 -> 74. **Critical path changes are separate**: B1 20 -> v2 18. See `M2T_CRITICAL_PATH_REVIEW.md` for the reading of these numbers.

## 2. Change log: original dependency set versus proposed

Original = B1 (the previous graph plus the acceptance edges and the persistence chain the ADG requires). Reasons, compatibility implications, migration paths and owner decisions are in `M2T_CRITICAL_PATH_REVIEW.md` section 5.

| Opt | Task | Original dependencies (corrected baseline B1) | Proposed dependencies (v2) |
|---|---|---|---|
| OPT-01 | API-02 | API-01, CORE-16, PROC-01, PROC-02, PROC-03, PROC-04, PROC-05, TEST-08 | API-01, PROC-01, PROC-04 |
| OPT-02 | ORCH-01 | API-01, API-02, API-03, API-04, API-05, CORE-16, PROC-01, PROC-02, PROC-03, PROC-04, PROC-05, TEST-08 | API-01, API-02, PROC-01 |
| OPT-02 | ORCH-02 | API-01, API-02, API-03, API-04, API-05, CORE-16, PROC-01, PROC-02, PROC-03, PROC-04, PROC-05, TEST-08 | API-02 |
| OPT-02 | ORCH-03 | API-01, API-02, API-03, API-04, API-05, CORE-10, CORE-16, PROC-01, PROC-02, PROC-03, PROC-04, PROC-05, TEST-08 | API-01, CORE-10, PROC-01 |
| OPT-02 | ORCH-04 | API-01, API-02, API-03, API-04, API-05, CORE-16, PROC-01, PROC-02, PROC-03, PROC-04, PROC-05, TEST-08 | API-02, API-04, PROC-02 |
| OPT-02 | ORCH-05 | API-01, API-02, API-03, API-04, API-05, CORE-08, CORE-16, PROC-01, PROC-02, PROC-03, PROC-04, PROC-05, TEST-08, XT-02 | API-02, API-03, CORE-08, XT-02 |
| OPT-02 | VOX-01 | API-01, API-02, API-03, API-04, API-05, CORE-16, PROC-01, PROC-02, PROC-03, PROC-04, PROC-05, TEST-03a, TEST-08 | API-02, PROC-01, TEST-03a |
| OPT-02 | VOX-02 | CORE-16, PROC-01, PROC-02, PROC-03, PROC-04, PROC-05, TEST-08 | PROC-01, PROC-02 |
| OPT-02 | VOX-05 | CORE-16, PROC-01, PROC-02, PROC-03, PROC-04, PROC-05, TEST-08 | PROC-01, PROC-02 |
| OPT-03 | NET-01 | API-02 | API-01, CORE-07, CORE-10 |
| OPT-04 | TGAP-03a | NET-02 | API-01, CORE-07, NET-01 |
| OPT-04 | SRV-04 | NET-02, SRV-03, WGEN-04 | SRV-03, TGAP-03a, WGEN-04 |
| OPT-04 | CLI-01 | CLI-00, CORE-06 | CLI-00, CORE-06, TGAP-03a |
| OPT-05 | SRV-01 | API-02, CORE-10, GOV-11, GOV-22, NET-01, NET-03, ORCH-01, ORCH-02, ORCH-03, ORCH-04, ORCH-05, PERS-01, PERS-02, PERS-03, PERS-06, PERS-08, TEST-03a, TEST-10, WGEN-03 | API-02, CORE-10, GOV-11, GOV-22, NET-01, NET-03, ORCH-01, ORCH-02, ORCH-03, ORCH-04, ORCH-05, PERS-01, PERS-02, PERS-03, PERS-08, WGEN-03 |
| OPT-08 | PERS-06 | API-04, CORE-09, GOV-11, PERS-05 | API-04, CORE-09, GOV-11, PERS-03 |
| OPT-08 | SRV-01/XI-01 | (no PERS-06 edge in v1) | PERS-06 added |
| OPT-06 | GM-12 | GM-10, GM-11, SRV-03, VOX-05 | GM-10, GM-11, VOX-05 |
| OPT-07 | NET-02 | CORE-07, GOV-22, NET-01 | removed from M2-T |
| OPT-07 | CLI-05 | CLI-01, NET-02, NET-03, SRV-04 | removed from M2-T |
| OPT-08 | TGAP-09 | - | PERS-04, PERS-05, PERS-06 |
| OPT-06 | XI-02 | ADP-03, CLI-01, CLI-02, CLI-05, GM-12, GM-14, GM-15, GM-16, INT-01, PERS-04, PERS-05, SRV-04, TEST-04, TEST-05, TEST-06, TEST-08, TEST-10, TGAP-03a, TGAP-04, TGAP-06, VOX-04, XI-01 | ADP-03, API-05, CLI-01, CLI-02, GM-12, GM-14, GM-15, GM-16, INT-01, SRV-03, SRV-04, TEST-05, TEST-06, TEST-08, TEST-10, TGAP-03a, TGAP-04, TGAP-06, VOX-04, XI-01 |

`XI-01` and `XI-02` also gain the acceptance edges that v1 omitted (`XI-01`: `VOX-01`, `WGEN-02`, `WGEN-04`, `TEST-09`, `TEST-02`, `PERS-06`, `TGAP-03a`; `XI-02`: `SRV-03`, `SRV-04`, `CLI-01`, `VOX-04`, `TEST-05`, `TEST-06`, `TEST-08`, `TEST-10`, `INT-01`, `API-05`).

## 3. Validation of v2 (generator output)

| Check | Result |
|---|---|
| no cycle in v2 | PASS |
| every dependency of v2 exists as a node of v2 | PASS |
| every M2-T task is reachable from the acceptance task XI-02 | PASS |
| M2-T tasks depend only on Pre and M2-T tasks | PASS |
| Pre tasks depend only on Pre tasks | PASS |
| no lower-layer task depends on a gameplay (GM-) task | PASS |
| NET-02 and CLI-05 are not referenced by any v2 task | PASS |
| strict chain length reproduced | PASS |
| official ADG file unchanged in the working tree | PASS |

## 4. Pre-implementation tasks (17)

Dependencies column: `G0*` = the eight G0 tasks `GOV-01`...`GOV-06`, `TEST-01`, `XT-01`. Class column is filled for M2-T tasks only.

| ID | Title | Track | Dependencies (v2) | Changed from v1/B1? | Owner | Acceptance criteria | Blocking decisions | Class |
|---|---|---|---|---|---|---|---|---|
| GOV-01 | Decide ACR-001/002/003, ACR-004 part A | Governance | - | - | Existing Owner (role) (Project owner) | every PS-1 prerequisite decision is Owner-Approved, Rejected or explicitly deferred with an extension point | - | - |
| GOV-02 | Owner-Approve P0-alpha; record Provisional Contracts PC-1 with assu... | Governance | GOV-01 | - | Existing Owner (role) (Project owner) | P0-alpha status Owner-Approved; PC-1 status Provisional with all mandatory fields | GOV-01 | - |
| GOV-03 | Decide owners for UNASSIGNED modules needed by the next stages | Governance | - | - | Existing Owner (role) (Project owner) | every module needed by PS-1..PS-5 has a named owner; others deferred with the stage that needs them | - | - |
| GOV-04 | Docs-only baseline on master | Governance | GOV-01 | - | Existing Owner (role) (Main Agent / Integration Owner) | no file of the spike code list is on master | GOV-01 | - |
| GOV-05 | CI SPECIFICATION document | Governance | GOV-01 | - | Existing Owner (role) (Main Agent / Integration Owner) | specification reviewed; every check has an owner and a pass condition | GOV-01 | - |
| GOV-06 | CI IMPLEMENTATION | Governance | GOV-04, GOV-05 | - | Existing Owner (role) (Agent I (QA and Benchmarks)) | a workflow run URL shows every required check green on an empty skeleton and red on seeded failures; branch... | GOV-04, GOV-05 | - |
| GOV-11 | Decide ACR-004 parts B | Governance | - | - | Existing Owner (role) (Project owner) | each part Owner-Approved, Rejected or deferred | - | - |
| GOV-21 | Owner-Approve the WorldGen contracts WG-01…WG-10 | Governance | - | - | Existing Owner (role) (Project owner) | Owner-Approved before the 11B tasks merge | - | - |
| GOV-22 | Owner-Approve the server-runtime contracts SR-01…SR-10 and the prot... | Governance | - | - | Existing Owner (role) (Project owner) | Owner-Approved before NET-02 and SRV-01 merge | - | - |
| SPK-PERS | Persistence durability spike | Technology | - | - | Existing Owner (role) (Agent H) | report states results per platform; never merged | - | - |
| TEST-01 | cube-testkit skeleton | Technology | GOV-05, GOV-06 | - | Existing Owner (role) (Agent I (QA and Benchmarks)) | used by at least two crates | GOV-05, GOV-06 | - |
| XT-01 | archcheck v0 | Technology | GOV-05, GOV-06 | - | Existing Owner (role) (Main Agent / Integration Owner) | catches every seeded violation; registered as a required status check | GOV-05, GOV-06 | - |
| GM-01 | Game Rules module design | Game | - | - | Owner Required (proposed: Gameplay Agent) | reviewed by the Integration Owner; Project Owner accepts the module | ODM-04, ODM-05 | - |
| GM-02 | Entity-model spike (ECS versus explicit domain model) | Game | - | - | Owner Required (proposed: Gameplay Agent) | finding states the choice and the reasons; no code is promoted | ODM-12 | - |
| GM-03 | M2 content tables | Game | - | - | Owner Required (proposed: Content/Design owner) | every item/creature used by FPS criteria is defined; values marked tunable | ODM-04 | - |
| GM-04 | Scenario library for FPS-01..FPS-17 (M2-T subset first) | Game | - | - | Existing Owner (role) Main Agent / Integration Owner, QA Agent I | every FPS criterion has a runnable scenario description | - | - |
| TGAP-05 | Runtime threading contract | Technology | - | - | Existing Owner (role) for drafting; Project Owner approves | every M2-T operation names its thread class | ODM-07 | - |

## 5. M2-T tasks (74)

| ID | Title | Track | Dependencies (v2) | Changed from v1/B1? | Owner | Acceptance criteria | Blocking decisions | Class |
|---|---|---|---|---|---|---|---|---|
| ADP-01 | Shrink the adapter to translation | Technology | CLI-01, CLI-02 | - | Owner Required (proposed: Engine Adapter Agent) | no algorithm left in the adapter | - | MUST_HAVE |
| ADP-02 | GDScript = UI and glue only; remove test code from the client | Technology | ADP-01 | - | Owner Required (proposed: Engine Adapter Agent) | AOD-07 and AOD-12 hold | - | MUST_HAVE |
| ADP-03 | Error wrapping, catch_unwind, panic policy test | Technology | ADP-01 | - | Owner Required (proposed: Engine Adapter Agent) | no abort across the FFI | - | MUST_HAVE |
| API-01 | Public types | Technology | CORE-07, CORE-10 | - | Existing Owner (role) (Main Agent / Integration Owner) | no behaviour in the crate | ODM-07 | MUST_HAVE |
| API-02 | Ports | Technology | API-01, PROC-01, PROC-04 | OPT-01 | Existing Owner (role) (Main Agent / Integration Owner) | backends compile without Orchestration | ODM-07 | MUST_HAVE |
| API-03 | AuthorizationContext | Technology | API-01 | - | Existing Owner (role) (Main Agent / Integration Owner) | mapping documented; LocalClient origin gives no extra rights | ODM-07 | MUST_HAVE |
| API-04 | Storage and recovery ports | Technology | API-01, CORE-09 | - | Existing Owner (role) (Main Agent / Integration Owner) | History and Persistence need no mutual dependency; recovery needs no WorldGen dependency | ODM-07 | MUST_HAVE |
| API-05 | Public-API snapshot test and contract-change tooling | Technology | API-01 | - | Existing Owner (role) (Main Agent / Integration Owner) | API diff fails CI without an accepted change request | ODM-07 | SHOULD_HAVE |
| CLI-00 | Client skeleton with a mock transport | Technology | API-01, CORE-10 | - | Owner Required (proposed: Client Runtime Agent) | not production; no authority | - | MUST_HAVE |
| CLI-01 | Chunk cache and interest view | Technology | CLI-00, CORE-06, TGAP-03a | OPT-04 | Owner Required (proposed: Client Runtime Agent) | read-only cache | - | MUST_HAVE |
| CLI-02 | Mesh scheduler | Technology | CLI-00, PROC-03, PROC-05, VOX-02 | - | Owner Required (proposed: Client Runtime Agent) | main-thread cost within the budget on the desktop profile | - | MUST_HAVE |
| CLI-04 | Input intents and prediction scaffolding | Technology | API-01, CLI-00 | - | Owner Required (proposed: Client Runtime Agent) | no authority | - | MUST_HAVE |
| CORE-01 | Layer model | Technology | G0* | - | Existing Owner (role) (Core Agent) | LayerData is a closed enum; no voxel-only type in the public API | ODM-07 | MUST_HAVE |
| CORE-02 | CellKey{layer,level,coord}, Region, coordinate conversions | Technology | G0* | - | Existing Owner (role) (Core Agent) | spike coordinate tests ported and green | ODM-07 | MUST_HAVE |
| CORE-03 | Channels, cell payload, cell state | Technology | CORE-01, CORE-02 | - | Existing Owner (role) (Core Agent) | absent data is never conflated with empty data | ODM-07 | MUST_HAVE |
| CORE-04 | GeometryTransform | Technology | CORE-01 | - | Existing Owner (role) (Core Agent) | no floating point in authoritative state (compile-time lint + test) | ODM-07 | MUST_HAVE |
| CORE-05 | RevisionToken | Technology | CORE-01, CORE-02 | - | Existing Owner (role) (Core Agent) | spike revision tests ported; overflow is a critical error, never wraps | ODM-07 | MUST_HAVE |
| CORE-06 | Commit | Technology | CORE-03, CORE-05, CORE-07 | - | Existing Owner (role) (Core Agent) | spike asset tests ported; loom green; all-or-nothing | ODM-07 | MUST_HAVE |
| CORE-07 | OperationId, ActorKind {User, System, Recovery}, error envelope | Technology | G0* | - | Existing Owner (role) (Core Agent) | every error code has a negative test; formats versioned per D-R8 | ODM-07 | MUST_HAVE |
| CORE-08 | Mutation boundary | Technology | CORE-06, CORE-07 | - | Existing Owner (role) (Core Agent) | a revision increment without a commit-log entry is impossible (test) | ODM-07 | MUST_HAVE |
| CORE-09 | System-commit and recovery ports | Technology | CORE-08 | - | Existing Owner (role) (Core Agent) | call-site audit passes; client-facing crates cannot name the handles | ODM-07 | MUST_HAVE |
| CORE-10 | CORE API FREEZE v0.1 | Technology | CORE-01, CORE-02, CORE-03, CORE-04, CORE-05, CORE-06, CORE-07 | - | Existing Owner (role) (Core Agent) | CI fails on any later public-API difference without an accepted change request | ODM-07 | MUST_HAVE |
| CORE-11 | Export/import views | Technology | CORE-03, CORE-05, CORE-06 | - | Existing Owner (role) (Core Agent) | no serialization format defined; hash algorithm and byte order fixed by the contract | ODM-07 | MUST_HAVE |
| CORE-16 | World identity value types WorldSeed and GeneratorVersion | Technology | CORE-10 | - | Existing Owner (role) (Core Agent) | additive to the frozen Core API; no WorldGen or Persistence logic in Core | ODM-07 | MUST_HAVE |
| INT-01 | Record contract status Implemented for the P0-alpha contracts; PC-1... | Technology | CORE-01, CORE-02, CORE-03, CORE-04, CORE-05, CORE-06, CORE-07, CORE-08, CORE-09, CORE-10, CORE-11, TEST-02, TEST-05, TEST-06, XT-02 | - | Existing Owner (role) (Main Agent / Integration Owner) | Implemented is recorded only with CI evidence; no contract is set Frozen | - | SHOULD_HAVE |
| NET-01 | Transport contract | Technology | API-01, CORE-07, CORE-10 | OPT-03 | Owner Required (proposed: Networking Agent) | no carrier named in the contract | - | ARCHITECTURE_COMPATIBILITY_ONLY |
| NET-03 | In-process transport | Technology | NET-01 | - | Owner Required (proposed: Networking Agent) | same semantics as the external transport | - | MUST_HAVE |
| ORCH-01 | Transaction lifecycle and rollback | Technology | API-01, API-02, PROC-01 | OPT-02 | Existing Owner (role) (Agent A) | no half-applied state | ODM-07 | MUST_HAVE |
| ORCH-02 | Backend registry and dispatch via capabilities | Technology | API-02 | OPT-02 | Existing Owner (role) (Agent A) | no backend import | ODM-07 | MUST_HAVE |
| ORCH-03 | Expected-token validation, conflict results, idempotent retry by Op... | Technology | API-01, CORE-10, PROC-01 | OPT-02 | Existing Owner (role) (Agent A) | retry never double-applies | ODM-07 | MUST_HAVE |
| ORCH-04 | Dirty propagation and observer notification | Technology | API-02, API-04, PROC-02 | OPT-02 | Existing Owner (role) (Agent A) | observers see post-commit state only | ODM-07 | MUST_HAVE |
| ORCH-05 | Authorisation and the single user commit path | Technology | API-02, API-03, CORE-08, XT-02 | OPT-02 | Existing Owner (role) (Agent A) | spike cube-editing behaviours reproduced through this path | ODM-07 | MUST_HAVE |
| PERS-01 | Cell/asset codec with schema versions, checksums, versioned Operati... | Technology | API-04, CORE-11 | - | Existing Owner (role) (Agent H) | every record carries a schema version; no format is Owner-Approved yet | ODM-03, ODM-07 | MUST_HAVE |
| PERS-02 | Manifest with atomic publish and directory fsync semantics | Technology | PERS-01, SPK-PERS | - | Existing Owner (role) (Agent H) | old or new manifest, never mixed; directory fsync verified | ODM-03, ODM-07 | MUST_HAVE |
| PERS-03 | Snapshot store | Technology | API-04, PERS-01, PERS-02 | - | Existing Owner (role) (Agent H) | snapshot invalid until verified | ODM-03, ODM-07 | MUST_HAVE |
| PERS-06 | Recovery of the user layer | Technology | API-04, CORE-09, GOV-11, PERS-03 | OPT-08 | Existing Owner (role) (Agent H) | Recovery actor only through the port; no WorldGen dependency | GOV-11, ODM-03, ODM-07 | MUST_HAVE |
| PERS-08 | World record | Technology | CORE-07, CORE-16, PERS-01 | - | Existing Owner (role) (Agent H) | untouched world = record only | ODM-03, ODM-07 | MUST_HAVE |
| PROC-01 | Generic snapshot with neighbour apron | Technology | CORE-06, CORE-10 | - | Existing Owner (role) (Processing Agent) | spike snapshot tests ported | ODM-02 | MUST_HAVE |
| PROC-02 | Provider registry and dependency graph | Technology | PROC-01 | - | Existing Owner (role) (Processing Agent) | derived stamp = input stamp set | ODM-02 | MUST_HAVE |
| PROC-03 | Executor, task state machine, priorities | Technology | CORE-10 | - | Existing Owner (role) (Processing Agent) | no async runtime dependency | ODM-02 | MUST_HAVE |
| PROC-04 | Cancellation and deadlines | Technology | PROC-03 | - | Existing Owner (role) (Processing Agent) | latency bound documented | ODM-02 | MUST_HAVE |
| PROC-05 | Stale rejection and DerivedSlot | Technology | PROC-01 | - | Existing Owner (role) (Processing Agent) | spike stale tests ported | ODM-02 | MUST_HAVE |
| SRV-01 | Server runtime | Technology | API-02, CORE-10, GOV-11, GOV-22, NET-01, NET-03, ORCH-01, ORCH-02, ORCH-03, ORCH-04, ORCH-05, PERS-01, PERS-02, PERS-03, PERS-06, PERS-08, WGEN-03 | OPT-05, OPT-08 | Owner Required (proposed: Server Runtime Agent) | AOD-22 rules; one runtime, two hosts | GOV-11, GOV-22 | MUST_HAVE |
| SRV-03 | Tick loop and single writer | Technology | ORCH-05, SRV-01 | - | Owner Required (proposed: Server Runtime Agent) | no concurrent commit paths | - | MUST_HAVE |
| SRV-04 | Interest management and replication | Technology | SRV-03, TGAP-03a, WGEN-04 | OPT-04 | Owner Required (proposed: Server Runtime Agent) | client cache equals server state | - | MUST_HAVE |
| TEST-02 | Golden-hash harness | Technology | CORE-11, TEST-01 | - | Existing Owner (role) (Agent I (QA and Benchmarks)) | hashes reproduce on the CI host | - | MUST_HAVE |
| TEST-03a | Backend conformance suite | Technology | API-02 | - | Existing Owner (role) (Agent I (QA and Benchmarks)) | each backend can run it | - | MUST_HAVE |
| TEST-05 | Core property and loom suites | Technology | CORE-06 | - | Existing Owner (role) (Agent I (QA and Benchmarks)) | suites run in CI and fail on seeded violations | - | SHOULD_HAVE |
| TEST-06 | Mutation-boundary negative tests available in PS-1 | Technology | CORE-08, CORE-09, XT-02 | - | Existing Owner (role) (Agent I (QA and Benchmarks)) | every seeded bypass fails CI | - | MUST_HAVE |
| TEST-08 | Processing concurrency and cancellation suites | Technology | PROC-03, PROC-05 | - | Existing Owner (role) (Agent I (QA and Benchmarks)) | no stale publication under any interleaving explored | - | SHOULD_HAVE |
| TEST-09 | Determinism suite on the CI host | Technology | TEST-02, WGEN-01 | - | Existing Owner (role) (Agent I (QA and Benchmarks)) | device equality is DG-2, not this task | - | MUST_HAVE |
| TEST-10 | Negative tests through the real Orchestration path | Technology | CORE-08, ORCH-05, XT-02 | - | Existing Owner (role) (Agent I (QA and Benchmarks)) | every seeded bypass fails CI | - | MUST_HAVE |
| VOX-01 | IGeometryBackend for Voxel | Technology | API-02, PROC-01, TEST-03a | OPT-02 | Existing Owner (role) (Agent B) | write sets carry expected cell revisions | ODM-07 | MUST_HAVE |
| VOX-02 | Faces provider | Technology | PROC-01, PROC-02 | OPT-02 | Existing Owner (role) (Agent B) | spike chunk_mesh tests ported | ODM-07 | MUST_HAVE |
| VOX-04 | Section dirty rule and incremental rebuild | Technology | VOX-02 | - | Existing Owner (role) (Agent B) | spike sections tests ported (11) | ODM-07 | MUST_HAVE |
| VOX-05 | Collision and LOD providers for voxel [pending O-1] | Technology | PROC-01, PROC-02 | OPT-02 | Existing Owner (role) (Agent B) | budgets documented | ODM-07 | MUST_HAVE |
| WGEN-01 | 11A Pure generator + version registry | Technology | CORE-07, CORE-10, CORE-16 | - | Owner Required (proposed: WorldGen Agent) | pinned hashes per version; no floats in world math | ODM-07 | MUST_HAVE |
| WGEN-02 | 11B Service | Technology | GOV-21, PROC-03, PROC-04, WGEN-01 | - | Owner Required (proposed: WorldGen Agent) | no stale result applied | GOV-21, ODM-07 | MUST_HAVE |
| WGEN-03 | 11B Materialiser through SystemCommit | Technology | CORE-09, GOV-11, GOV-21, WGEN-01 | - | Owner Required (proposed: WorldGen Agent) | only writer of generated data | GOV-11, GOV-21, ODM-07 | MUST_HAVE |
| WGEN-04 | 11B Interest handling and eviction | Technology | API-04, WGEN-02, WGEN-03 | - | Owner Required (proposed: WorldGen Agent) | server-driven only | ODM-07 | MUST_HAVE |
| XT-02 | archcheck v1 | Technology | CORE-08, CORE-09 | - | Existing Owner (role) (Main Agent / Integration Owner) | every seeded direct-write violation is caught; registered as a required status check; runs on every PR | - | MUST_HAVE |
| GM-10 | Game Rules crate skeleton and ports | Game | API-02, CORE-10, GM-01, GM-02 | - | Owner Required (proposed: Gameplay Agent) | builds without godot; archcheck rule for its dependencies passes; ports traced to existing contracts | ODM-04, ODM-05, ODM-07 | MUST_HAVE |
| GM-11 | Player entity and state | Game | GM-10 | - | Owner Required (proposed: Gameplay Agent) | state is server-owned; serialisable through the record store | ODM-05 | MUST_HAVE |
| GM-12 | Player controller (movement, gravity, jump, step-up, collision) | Game | GM-10, GM-11, VOX-05 | OPT-06 | Owner Required (proposed: Gameplay Agent) | scripted walk across chunk borders and negative coordinates; jump; step; no tunnelling at the tested speeds | ODM-05, ODM-11 | MUST_HAVE |
| GM-13 | World interaction (reach, pick, destroy, place) | Game | API-03, GM-10, ORCH-05, TGAP-01 | - | Owner Required (proposed: Gameplay Agent) | every edit is a UserCommit; out-of-reach requests rejected; negative-coordinate cases | ODM-05 | MUST_HAVE |
| GM-14 | Input mapping and third-person camera | Game | ADP-01, CLI-04 | - | Owner Required (proposed: Engine Adapter Agent; option: a separate Presentation owner, ODM-04) | keyboard and mouse mapped to intents; no rules in script code | ODM-04 | MUST_HAVE |
| GM-15 | Menus and world selection | Game | ADP-02, GM-10, PERS-08 | - | Owner Required (proposed: Engine Adapter Agent; option: a separate Presentation owner, ODM-04) | FPS-01/02/10 steps possible through the UI | ODM-04 | MUST_HAVE |
| GM-16 | Placeholder player character | Game | - | - | Owner Required (proposed: Art/Audio owner) | visible in the client; replaceable without code change | ODM-04 | SHOULD_HAVE |
| TGAP-01 | Voxel ray and spatial query API | Technology | CORE-02, CORE-03, PROC-01 | - | Owner Required (query-API home open; proposed: Processing Agent) | correct on chunk borders and negative coordinates | ODM-02, ODM-05 | MUST_HAVE |
| TGAP-03a | Protocol messages for intents, player state and chunk data | Technology | API-01, CORE-07, NET-01 | OPT-04 | Owner Required (proposed: Networking Agent) | intents up, state down over the in-process transport | ODM-04 | MUST_HAVE |
| TGAP-04 | Desktop build and runtime integration | Technology | ADP-01, GOV-06 | - | Existing Owner (role) | a clean checkout produces the playable build | ODM-09 | MUST_HAVE |
| TGAP-06 | Stability harness | Technology | PERS-03, XI-01 | - | Existing Owner (role) | harness runs to completion and reports; thresholds only by owner approval | ODM-08 | MUST_HAVE |
| XI-01 | Headless integration scenario (create by seed, edit, save, reload) | Integration | CORE-11, GM-13, ORCH-05, PERS-03, PERS-06, PROC-05, SRV-01, TEST-02, TEST-09, TGAP-03a, VOX-01, WGEN-02, WGEN-03, WGEN-04 | OPT-08 | Shared Responsibility (Integration Owner: Main Agent; module owners contribute) | FPS-02, 09, 10, 11 (world part) pass headless | - | MUST_HAVE |
| XI-02 | M2-T client integration and demo build | Integration | ADP-03, API-05, CLI-01, CLI-02, GM-12, GM-14, GM-15, GM-16, INT-01, SRV-03, SRV-04, TEST-05, TEST-06, TEST-08, TEST-10, TGAP-03a, TGAP-04, TGAP-06, VOX-04, XI-01 | OPT-06 | Shared Responsibility (Integration Owner: Main Agent) | M2-T criteria of M2_SCOPE_REVIEW.md demonstrated | ODM-06 | MUST_HAVE |

## 6. M2 additions (27)

| ID | Title | Track | Dependencies (v2) | Changed from v1/B1? | Owner | Acceptance criteria | Blocking decisions | Class |
|---|---|---|---|---|---|---|---|---|
| ITG-01 | Integration test | Technology | ORCH-01, ORCH-02, ORCH-03, ORCH-04, ORCH-05, PERS-04, TEST-03a, TEST-10 | - | Existing Owner (role) (Agent I (QA and Benchmarks)) | replay equals live state for the spike scenarios | - | - |
| ITG-02 | Integration test | Technology | GOV-11, ITG-01, PERS-01, PERS-02, PERS-03, PERS-04, PERS-05, PERS-06, PERS-08, QA-P2, TEST-04, WGEN-04 | - | Existing Owner (role) (Agent I (QA and Benchmarks)) | replay of the spike scenarios equals live state | GOV-11 | - |
| PERS-04 | Mutation journal and delta store | Technology | API-04, CORE-05, CORE-07, PERS-01 | - | Existing Owner (role) (Agent H) | survives sudden termination; replay yields identical commit sequence | ODM-03, ODM-07 | - |
| PERS-05 | Compaction protocol | Technology | PERS-02, PERS-03, PERS-04, SPK-PERS, TEST-04 | - | Existing Owner (role) (Agent H) | prefix truncated only after verification | ODM-03, ODM-07 | - |
| QA-P2 | Persistence crash-recovery matrix on desktop | Technology | PERS-05, PERS-06, TEST-04 | - | Existing Owner (role) (Agent I (QA and Benchmarks)) | all crash points recover; no unverified snapshot ever becomes sole truth | - | - |
| TEST-04 | Fault-injecting filesystem and crash-point runner | Technology | TEST-01 | - | Existing Owner (role) (Agent I (QA and Benchmarks)) | can crash after any numbered I/O step | - | - |
| GM-20 | Item definitions and data loader | Game | GM-03, GM-10 | - | Owner Required (proposed: Gameplay Agent) | all M2 items load; malformed data is rejected with a coded error | ODM-13 | - |
| GM-21 | Inventory | Game | GM-11, GM-20 | - | Owner Required (proposed: Gameplay Agent) | invariants: no duplication, no loss under repeated requests | ODM-05 | - |
| GM-22 | Equipment and derived stats | Game | GM-21 | - | Owner Required (proposed: Gameplay Agent) | equipping changes attack stat; unequip restores | - | - |
| GM-23 | Health and damage model | Game | GM-11 | - | Owner Required (proposed: Gameplay Agent) | damage never below minimum; death emitted once | - | - |
| GM-24 | Combat (attack, hit detection, cooldown) | Game | GM-12, GM-23, VOX-05 | - | Owner Required (proposed: Gameplay Agent) | FPS-06: win and lose scenarios | - | - |
| GM-25 | Creature framework (kinds, spawn, despawn, identifiers) | Game | GM-10, GM-11, SRV-03, TGAP-08 | - | Owner Required (proposed: Gameplay Agent) | same seed gives the same spawns; no creature outlives its chunk | ODM-05 | - |
| GM-26 | Creature AI (hostile chaser, passive wanderer) | Game | GM-12, GM-25, TGAP-01 | - | Owner Required (proposed: Gameplay Agent) | FPS-05: both creatures observable; hostile attacks within its rules | - | - |
| GM-27 | Loot (seeded drops) | Game | GM-20, GM-23, GM-25 | - | Owner Required (proposed: Gameplay Agent) | same seed and event give the same drop | - | - |
| GM-28 | Death and respawn | Game | GM-11, GM-23 | - | Owner Required (proposed: Gameplay Agent) | FPS-06 lose case | - | - |
| GM-29 | Gameplay persistence (player record) | Game | GM-11, GM-21, GM-22, TGAP-02 | - | Owner Required (proposed: Gameplay Agent) | FPS-11: state identical after reload | ODM-07 | - |
| GM-30 | Gameplay state and event emission | Game | GM-21, GM-24, GM-25, TGAP-03b | - | Owner Required (proposed: Gameplay Agent) | server state reaches the protocol layer with stable identifiers | ODM-05 | - |
| GM-31 | Game HUD and inventory UI | Game | ADP-02, GM-33 | - | Owner Required (proposed: Engine Adapter Agent; option: a separate Presentation owner, ODM-04) | FPS-07/08 visible to the player | - | - |
| GM-32 | Placeholder creature models and item icons | Game | GM-03 | - | Owner Required (proposed: Art/Audio owner) | visible in client | - | - |
| GM-33 | Client views for entities, inventory and events | Game | CLI-01, GM-30, TGAP-03b | - | Owner Required (proposed: Client Runtime Agent) | client shows server state only; no local authority | ODM-04 | - |
| TGAP-02 | Game record store (opaque, versioned records next to the world) | Technology | API-04, PERS-01, PERS-02, PERS-08 | - | Existing Owner (role) | records survive save/load and recovery; persistence imports no gameplay type | ODM-07 | - |
| TGAP-03b | Protocol messages for entities, inventory and events | Technology | GM-10, TGAP-03a | - | Owner Required (proposed: Networking Agent) | covers creature snapshots, inventory views, combat events | ODM-04 | - |
| TGAP-07 | Second terrain variant, boundary and material set | Technology | CORE-16, WGEN-01 | - | Owner Required (proposed: WorldGen Agent) | FPS-02/04: hashes pinned; no seams at the variant boundary | ODM-04 | - |
| TGAP-08 | Spawn-candidate step in world generation | Technology | WGEN-01, WGEN-03 | - | Owner Required (proposed: WorldGen Agent) | terrain hash independent of spawn step | ODM-04 | - |
| TGAP-09 | Journal-tail recovery (M2 extension of PERS-06) | Technology | PERS-04, PERS-05, PERS-06 | OPT-08 | Existing Owner (role) (Agent H) | snapshot + tail equals full replay; truncated tail handled; double recovery idempotent (WP-08 tests) | GOV-11, ODM-03 | MUST_HAVE |
| XI-03 | M2 integration | Integration | GM-22, GM-26, GM-27, GM-28, GM-29, GM-31, GM-32, GM-33, ITG-01, ITG-02, PERS-06, QA-P2, TGAP-07, TGAP-09, XI-02 | - | Shared Responsibility (Integration Owner: Main Agent) | FPS-01..FPS-17 demonstrated | ODM-06 | - |
| XI-04 | Outside-tester session and M2 evidence package | Integration | XI-03 | - | Existing Owner (role) coordinates; Project Owner accepts | FPS-14 recorded | ODM-06 | - |

## 7. Not in the graph

Everything else in the ADG (History, tools, Composite, Resampling, SDF and mesh backends, greedy mesher, LOD, dedicated host, external transport, sessions and authentication integration, reconnect, permissions, device gates, full QA) remains in the ADG as before and is not part of M2-T or M2.
